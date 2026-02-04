use tauri::{command, AppHandle, State};

use crate::models::{McpServerConfig, McpServerStatus, ResourceDefinition, ToolDefinition};
use crate::state::AppState;

async fn persist_config(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    let config = state.mcp_service.get_config().await;
    let path = crate::commands::mcp_config::build_mcp_config_path(app)?;
    crate::utils::save_json(&path, &config)?;
    Ok(())
}

#[command]
pub async fn mcp_list_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, String> {
    Ok(state.mcp_service.list_servers().await)
}

#[command]
pub async fn mcp_add_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    state.mcp_service.add_server(server).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_update_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    state.mcp_service.update_server(server).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_remove_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.mcp_service.remove_server(&id).await?;
    persist_config(&app, &state).await
}

#[command]
pub async fn mcp_connect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.mcp_service.connect(&id).await
}

#[command]
pub async fn mcp_disconnect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.mcp_service.disconnect(&id).await
}

#[command]
pub async fn mcp_status(
    state: State<'_, AppState>,
    id: Option<String>,
) -> Result<Vec<McpServerStatus>, String> {
    Ok(state.mcp_service.status(id).await)
}

#[command]
pub async fn mcp_tools_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ToolDefinition>, String> {
    state.mcp_service.tools_list(&id).await
}

#[command]
pub async fn mcp_tools_call(
    state: State<'_, AppState>,
    id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    state
        .mcp_service
        .tools_call(&id, &tool_name, arguments)
        .await
}

#[command]
pub async fn mcp_resources_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ResourceDefinition>, String> {
    state.mcp_service.resources_list(&id).await
}

#[command]
pub async fn mcp_resources_read(
    state: State<'_, AppState>,
    id: String,
    uri: String,
) -> Result<serde_json::Value, String> {
    state.mcp_service.resources_read(&id, &uri).await
}
