use std::path::Path;
use tauri::{command, AppHandle, State};

use crate::models::{
    McpConfig, McpServerConfig, McpServerStatus, ResourceDefinition, ToolDefinition,
};
use crate::services::mcp::McpService;
use crate::state::AppState;

async fn persist_config(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    let config = state.mcp_service.get_config().await;
    let path = crate::commands::mcp_config::build_mcp_config_path(app)?;
    persist_config_to_path(&path, &config)
}

#[command]
pub async fn mcp_list_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, String> {
    Ok(mcp_list_servers_with_service(&state.mcp_service).await)
}

#[command]
pub async fn mcp_add_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    mcp_add_server_with_service(&state.mcp_service, server).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_update_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    mcp_update_server_with_service(&state.mcp_service, server).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_remove_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    mcp_remove_server_with_service(&state.mcp_service, id).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_connect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    mcp_connect_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn mcp_disconnect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    mcp_disconnect_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn mcp_status(
    state: State<'_, AppState>,
    id: Option<String>,
) -> Result<Vec<McpServerStatus>, String> {
    Ok(mcp_status_with_service(&state.mcp_service, id).await)
}

#[command]
pub async fn mcp_tools_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ToolDefinition>, String> {
    mcp_tools_list_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn mcp_tools_call(
    state: State<'_, AppState>,
    id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    mcp_tools_call_with_service(&state.mcp_service, id, tool_name, arguments).await
}

#[command]
pub async fn mcp_resources_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ResourceDefinition>, String> {
    mcp_resources_list_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn mcp_resources_read(
    state: State<'_, AppState>,
    id: String,
    uri: String,
) -> Result<serde_json::Value, String> {
    mcp_resources_read_with_service(&state.mcp_service, id, uri).await
}

pub fn persist_config_to_path(path: &Path, config: &McpConfig) -> Result<(), String> {
    crate::utils::save_json(path, config)
}

pub async fn mcp_list_servers_with_service(service: &McpService) -> Vec<McpServerConfig> {
    service.list_servers().await
}

pub async fn mcp_add_server_with_service(
    service: &McpService,
    server: McpServerConfig,
) -> Result<(), String> {
    service.add_server(server).await
}

pub async fn mcp_update_server_with_service(
    service: &McpService,
    server: McpServerConfig,
) -> Result<(), String> {
    service.update_server(server).await
}

pub async fn mcp_remove_server_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.remove_server(&id).await
}

pub async fn mcp_connect_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.connect(&id).await
}

pub async fn mcp_disconnect_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.disconnect(&id).await
}

pub async fn mcp_status_with_service(
    service: &McpService,
    id: Option<String>,
) -> Vec<McpServerStatus> {
    service.status(id).await
}

pub async fn mcp_tools_list_with_service(
    service: &McpService,
    id: String,
) -> Result<Vec<ToolDefinition>, String> {
    service.tools_list(&id).await
}

pub async fn mcp_tools_call_with_service(
    service: &McpService,
    id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    service.tools_call(&id, &tool_name, arguments).await
}

pub async fn mcp_resources_list_with_service(
    service: &McpService,
    id: String,
) -> Result<Vec<ResourceDefinition>, String> {
    service.resources_list(&id).await
}

pub async fn mcp_resources_read_with_service(
    service: &McpService,
    id: String,
    uri: String,
) -> Result<serde_json::Value, String> {
    service.resources_read(&id, &uri).await
}
