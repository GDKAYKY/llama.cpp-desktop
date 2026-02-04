use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager, State};

use crate::models::McpConfig;
use crate::state::AppState;

fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?;

    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(app_dir)
}

pub fn build_mcp_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut path = get_config_path(app)?;
    path.push("mcp.json");
    Ok(path)
}

pub fn load_mcp_config_file(app: &AppHandle) -> Result<McpConfig, String> {
    let path = build_mcp_config_path(app)?;
    if !path.exists() {
        return Ok(McpConfig::default());
    }
    crate::utils::read_json(&path)
}

#[command]
pub async fn load_mcp_config(app: AppHandle, state: State<'_, AppState>) -> Result<McpConfig, String> {
    let config = load_mcp_config_file(&app)?;
    state.mcp_service.set_config(config.clone()).await;
    Ok(config)
}

#[command]
pub async fn save_mcp_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: McpConfig,
) -> Result<(), String> {
    let path = build_mcp_config_path(&app)?;
    crate::utils::save_json(&path, &config)?;
    state.mcp_service.set_config(config).await;
    Ok(())
}

#[command]
pub async fn reset_mcp_config(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<McpConfig, String> {
    let config = McpConfig::default();
    let path = build_mcp_config_path(&app)?;
    crate::utils::save_json(&path, &config)?;
    state.mcp_service.set_config(config.clone()).await;
    Ok(config)
}

#[command]
pub fn get_mcp_config_path_string(app: AppHandle) -> Result<String, String> {
    let path = build_mcp_config_path(&app)?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid config path".to_string())
}
