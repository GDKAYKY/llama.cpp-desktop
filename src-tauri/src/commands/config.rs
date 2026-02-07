use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Manager};

use crate::models::{AppConfig, ConfigError};

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
    let path = build_config_file_path_from_dir(get_config_path(app)?);
    Ok(path)
}

pub fn get_config(app: &AppHandle) -> Result<AppConfig, String> {
    let config_path = build_config_file_path(app).map_err(|e| e.to_string())?;
    get_config_from_path(&config_path)
}

#[command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    get_config(&app)
}

#[command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let config_path = build_config_file_path(&app).map_err(|e| e.to_string())?;
    save_config_to_path(&config_path, &config)
}

#[command]
pub async fn reset_config(app: AppHandle) -> Result<AppConfig, String> {
    let config_path = build_config_file_path(&app).map_err(|e| e.to_string())?;
    reset_config_at_path(&config_path)
}

#[command]
pub fn get_config_path_string(app: AppHandle) -> Result<String, String> {
    let path = build_config_file_path(&app).map_err(|e| e.to_string())?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid config path".to_string())
}

pub fn build_config_file_path_from_dir(mut app_dir: PathBuf) -> PathBuf {
    app_dir.push("config.json");
    app_dir
}

pub fn get_config_from_path(config_path: &Path) -> Result<AppConfig, String> {
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    crate::utils::read_json(config_path)
}

pub fn save_config_to_path(config_path: &Path, config: &AppConfig) -> Result<(), String> {
    crate::utils::save_json(config_path, config)
}

pub fn reset_config_at_path(config_path: &Path) -> Result<AppConfig, String> {
    let config = AppConfig::default();
    save_config_to_path(config_path, &config)?;
    Ok(config)
}
