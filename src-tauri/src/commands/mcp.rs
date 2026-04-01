use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tauri::{command, AppHandle, State};

use crate::models::{
    McpConfig, McpServerConfig, McpServerStatus, McpTransport, ResourceDefinition, ToolDefinition,
};
use crate::services::mcp::McpService;
use crate::services::orchestrator::ChatOrchestrator;
use crate::state::AppState;

async fn persist_config(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    let config = state.mcp_service.get_config().await;
    let path = crate::commands::mcp_config::build_mcp_config_path(app)?;
    persist_config_to_path(&path, &config)
}

#[command]
pub async fn list_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, String> {
    Ok(list_servers_with_service(&state.mcp_service).await)
}

#[command]
pub async fn add_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    add_server_with_service(&state.mcp_service, server).await?;
    persist_config(&app, &state).await?;
    spawn_refresh_capabilities(state.orchestrator.clone());
    Ok(())
}

#[command]
pub async fn update_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: McpServerConfig,
) -> Result<(), String> {
    update_server_with_service(&state.mcp_service, server).await?;
    persist_config(&app, &state).await?;
    spawn_refresh_capabilities(state.orchestrator.clone());
    Ok(())
}

#[command]
pub async fn remove_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    remove_server_with_service(&state.mcp_service, id).await?;
    persist_config(&app, &state).await?;
    spawn_refresh_capabilities(state.orchestrator.clone());
    Ok(())
}

#[command]
pub async fn connect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    connect_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn disconnect(state: State<'_, AppState>, id: String) -> Result<(), String> {
    disconnect_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn status(
    state: State<'_, AppState>,
    id: Option<String>,
) -> Result<Vec<McpServerStatus>, String> {
    Ok(status_with_service(&state.mcp_service, id).await)
}

#[command]
pub async fn list_tools(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ToolDefinition>, String> {
    list_tools_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn call_tools(
    state: State<'_, AppState>,
    id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    call_tools_with_service(&state.mcp_service, id, tool_name, arguments).await
}

#[command]
pub async fn list_resources(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ResourceDefinition>, String> {
    list_resources_with_service(&state.mcp_service, id).await
}

#[command]
pub async fn read_resources(
    state: State<'_, AppState>,
    id: String,
    uri: String,
) -> Result<serde_json::Value, String> {
    read_resources_with_service(&state.mcp_service, id, uri).await
}

pub fn persist_config_to_path(path: &Path, config: &McpConfig) -> Result<(), String> {
    crate::utils::save_json(path, config)
}

pub async fn list_servers_with_service(service: &McpService) -> Vec<McpServerConfig> {
    service.list_servers().await
}

pub async fn add_server_with_service(
    service: &McpService,
    server: McpServerConfig,
) -> Result<(), String> {
    service.add_server(server).await
}

pub async fn update_server_with_service(
    service: &McpService,
    server: McpServerConfig,
) -> Result<(), String> {
    service.update_server(server).await
}

pub async fn remove_server_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.remove_server(&id).await
}

pub async fn connect_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.connect(&id).await
}

pub async fn disconnect_with_service(service: &McpService, id: String) -> Result<(), String> {
    service.disconnect(&id).await
}

pub async fn status_with_service(service: &McpService, id: Option<String>) -> Vec<McpServerStatus> {
    service.status(id).await
}

pub async fn list_tools_with_service(
    service: &McpService,
    id: String,
) -> Result<Vec<ToolDefinition>, String> {
    service.tools_list(&id).await
}

pub async fn call_tools_with_service(
    service: &McpService,
    id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    service.call_tools(&id, &tool_name, arguments).await
}

pub async fn list_resources_with_service(
    service: &McpService,
    id: String,
) -> Result<Vec<ResourceDefinition>, String> {
    service.resources_list(&id).await
}

pub async fn read_resources_with_service(
    service: &McpService,
    id: String,
    uri: String,
) -> Result<serde_json::Value, String> {
    service.resources_read(&id, &uri).await
}

#[command]
pub async fn refresh_mcp_capabilities(state: State<'_, AppState>) -> Result<(), String> {
    state.orchestrator.refresh_capabilities().await
}

#[command]
pub async fn parse_config(payload: Value) -> Result<McpServerConfig, String> {
    parse_mcp_server_from_value(&payload)
        .ok_or_else(|| "Não encontrei uma configuração MCP válida.".to_string())
}

#[command]
pub async fn import_config(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: Value,
) -> Result<McpServerConfig, String> {
    let server = parse_mcp_server_from_value(&payload)
        .ok_or_else(|| "Não encontrei uma configuração MCP válida.".to_string())?;
    add_server_with_service(&state.mcp_service, server.clone()).await?;
    persist_config(&app, &state).await?;
    spawn_refresh_capabilities(state.orchestrator.clone());
    Ok(server)
}

/// Spawn a background task to refresh the capability registry.
/// Non-blocking — fire and forget.
fn spawn_refresh_capabilities(orchestrator: ChatOrchestrator) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = orchestrator.refresh_capabilities().await {
            eprintln!("[MCP] Failed to refresh capabilities: {}", e);
        }
    });
}

fn parse_mcp_server_from_value(value: &Value) -> Option<McpServerConfig> {
    if let Some(arr) = value.as_array() {
        return arr.first().and_then(parse_mcp_server_from_value);
    }

    if let Some(obj) = value.as_object() {
        if let Some(servers) = obj.get("servers").and_then(|v| v.as_array()) {
            return servers.first().and_then(parse_mcp_server_from_value);
        }

        if let Some(map) = obj.get("mcpServers").and_then(|v| v.as_object()) {
            if let Some((key, raw)) = map.iter().next() {
                if let Some(server_obj) = raw.as_object() {
                    return Some(build_server_from_legacy_map(key, server_obj));
                }
            }
        }
    }

    if let Ok(config) = serde_json::from_value::<McpServerConfig>(value.clone()) {
        return Some(config);
    }

    if let Some(obj) = value.as_object() {
        return build_server_from_object(obj);
    }

    None
}

fn build_server_from_object(obj: &serde_json::Map<String, Value>) -> Option<McpServerConfig> {
    let id = obj.get("id")?.as_str()?.to_string();
    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();
    let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let transport = parse_transport(obj.get("transport").or_else(|| obj.get("type")))
        .or_else(|| infer_transport(obj))?;
    Some(McpServerConfig {
        id,
        name,
        enabled,
        transport,
        command: obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        args: parse_string_list(obj.get("args")),
        cwd: obj
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        env: parse_string_map(obj.get("env")),
        url: obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        headers: parse_string_map(obj.get("headers")),
        tool_allowlist: parse_string_list(obj.get("tool_allowlist")),
        resource_allowlist: parse_string_list(obj.get("resource_allowlist")),
    })
}

fn build_server_from_legacy_map(
    key: &str,
    obj: &serde_json::Map<String, Value>,
) -> McpServerConfig {
    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(key)
        .to_string();
    let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let transport = parse_transport(obj.get("transport").or_else(|| obj.get("type")))
        .or_else(|| infer_transport(obj))
        .unwrap_or(McpTransport::HttpSse);
    McpServerConfig {
        id: key.to_string(),
        name,
        enabled,
        transport,
        command: obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        args: parse_string_list(obj.get("args")),
        cwd: obj
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        env: parse_string_map(obj.get("env")),
        url: obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        headers: parse_string_map(obj.get("headers")),
        tool_allowlist: parse_string_list(obj.get("tool_allowlist")),
        resource_allowlist: parse_string_list(obj.get("resource_allowlist")),
    }
}

fn parse_transport(value: Option<&Value>) -> Option<McpTransport> {
    let raw = value?.as_str()?.to_lowercase();
    match raw.as_str() {
        "stdio" | "local" => Some(McpTransport::Stdio),
        "http_sse" | "http" | "sse" => Some(McpTransport::HttpSse),
        _ => None,
    }
}

fn infer_transport(obj: &serde_json::Map<String, Value>) -> Option<McpTransport> {
    if obj.get("command").and_then(|v| v.as_str()).is_some() {
        return Some(McpTransport::Stdio);
    }
    if obj.get("url").and_then(|v| v.as_str()).is_some() {
        return Some(McpTransport::HttpSse);
    }
    None
}

fn parse_string_list(value: Option<&Value>) -> Option<Vec<String>> {
    match value {
        Some(Value::Array(items)) => {
            let list: Vec<String> = items
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if list.is_empty() {
                None
            } else {
                Some(list)
            }
        }
        Some(Value::String(s)) => Some(vec![s.clone()]),
        _ => None,
    }
}

fn parse_string_map(value: Option<&Value>) -> Option<HashMap<String, String>> {
    let obj = value?.as_object()?;
    let mut map = HashMap::new();
    for (key, val) in obj {
        if let Some(s) = val.as_str() {
            map.insert(key.clone(), s.to_string());
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}
