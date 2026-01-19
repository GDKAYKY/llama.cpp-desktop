use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub models_directory: Option<String>,
    pub llamaPath: Option<String>,
    pub theme: String,
    pub language: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub auto_save_chat: bool,
    pub chat_history_limit: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            models_directory: None,
            llamaPath: None,
            theme: "dark".to_string(),
            language: "en".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            auto_save_chat: true,
            chat_history_limit: 50,
        }
    }
}

fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?;

    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(app_dir.join("config.json"))
}

#[command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let config_path = get_config_path(&app)?;

    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

#[command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path(&app)?;

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[command]
pub async fn reset_config(app: AppHandle) -> Result<AppConfig, String> {
    let config = AppConfig::default();
    save_config(app, config.clone()).await?;
    Ok(config)
}

#[command]
pub async fn get_config_path_string(app: AppHandle) -> Result<String, String> {
    let path = get_config_path(&app)?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid config path".to_string())
}
