use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub models_directory: Option<String>,
    pub llama_directory: Option<String>,
    pub theme: String,
    pub language: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub auto_save_chat: bool,
    pub chat_history_limit: u32,
    pub server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigError {
    Io(String),
    NotFound,
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::NotFound => write!(f, "Config not found"),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            models_directory: Some("E:\\models".to_string()),
            llama_directory: Some("E:\\src\\llama_cpp".to_string()),
            theme: "dark".to_string(),
            language: "en".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            auto_save_chat: true,
            chat_history_limit: 50,
            server_port: 8080,
        }
    }
}

fn get_config_path(app: &AppHandle) -> Result<PathBuf, ConfigError> {
    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| ConfigError::Io(format!("Failed to get app config directory: {}", e)))?;

    fs::create_dir_all(&app_dir)
        .map_err(|e| ConfigError::Io(format!("Failed to create config directory: {}", e)))?;

    Ok(app_dir)
}

pub fn build_config_file_path(app: &AppHandle) -> Result<PathBuf, ConfigError> {
    let mut path = get_config_path(app)?;
    path.push("config.json");
    Ok(path)
}

pub fn get_config(app: &AppHandle) -> Result<AppConfig, String> {
    let config_path = build_config_file_path(app).map_err(|e| e.to_string())?;

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
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    get_config(&app)
}

#[command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let config_path = build_config_file_path(&app).map_err(|e| e.to_string())?;

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
pub fn get_config_path_string(app: AppHandle) -> Result<String, String> {
    let path = build_config_file_path(&app).map_err(|e| e.to_string())?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid config path".to_string())
}
