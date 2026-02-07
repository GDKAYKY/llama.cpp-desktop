use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::models::{
    McpConfig, McpServerConfig, McpServerStatus, McpTransport, ResourceDefinition,
    ToolDefinition,
};
use crate::services::mcp::client::McpClient;

pub struct McpService {
    config: Arc<Mutex<McpConfig>>,
    connections: Arc<Mutex<HashMap<String, McpConnection>>>,
    stdio_connector: StdioConnector,
}

type StdioConnectFuture = Pin<Box<dyn Future<Output = Result<McpClient, String>> + Send>>;
type StdioConnector = Arc<
    dyn Fn(String, Vec<String>, Option<String>, Option<HashMap<String, String>>) -> StdioConnectFuture
        + Send
        + Sync,
>;

impl McpService {
    pub fn new(config: McpConfig) -> Self {
        Self::new_with_stdio_connector(config, |command, args, cwd, env| {
            Box::pin(McpClient::connect_stdio_owned(command, args, cwd, env))
        })
    }

    pub fn new_with_stdio_connector<F, Fut>(config: McpConfig, connector: F) -> Self
    where
        F: Fn(String, Vec<String>, Option<String>, Option<HashMap<String, String>>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Result<McpClient, String>> + Send + 'static,
    {
        let stdio_connector: StdioConnector = Arc::new(move |command, args, cwd, env| {
            Box::pin(connector(command, args, cwd, env))
        });
        Self {
            config: Arc::new(Mutex::new(config)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            stdio_connector,
        }
    }

    pub async fn set_config(&self, config: McpConfig) {
        let mut cfg = self.config.lock().await;
        *cfg = config;
    }

    pub async fn get_config(&self) -> McpConfig {
        self.config.lock().await.clone()
    }

    pub async fn list_servers(&self) -> Vec<McpServerConfig> {
        self.config.lock().await.servers.clone()
    }

    pub async fn add_server(&self, server: McpServerConfig) -> Result<(), String> {
        let mut cfg = self.config.lock().await;
        if cfg.servers.iter().any(|s| s.id == server.id) {
            return Err("Server with same id already exists".to_string());
        }
        cfg.servers.push(server);
        Ok(())
    }

    pub async fn update_server(&self, server: McpServerConfig) -> Result<(), String> {
        let mut cfg = self.config.lock().await;
        let Some(existing) = cfg.servers.iter_mut().find(|s| s.id == server.id) else {
            return Err("Server not found".to_string());
        };
        *existing = server;
        Ok(())
    }

    pub async fn remove_server(&self, id: &str) -> Result<(), String> {
        let mut cfg = self.config.lock().await;
        cfg.servers.retain(|s| s.id != id);
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.remove(id) {
            conn.client.shutdown().await;
        }
        Ok(())
    }

    pub async fn connect(&self, id: &str) -> Result<(), String> {
        let server = {
            let cfg = self.config.lock().await;
            cfg.servers
                .iter()
                .find(|s| s.id == id)
                .cloned()
                .ok_or_else(|| "Server not found".to_string())?
        };

        if !server.enabled {
            return Err("Server is disabled".to_string());
        }

        let client = match server.transport {
            McpTransport::Stdio => {
                let command = server
                    .command
                    .as_ref()
                    .ok_or_else(|| "Missing command".to_string())?;
                let args = server.args.clone().unwrap_or_default();
                (self.stdio_connector)(
                    command.clone(),
                    args,
                    server.cwd.clone(),
                    server.env.clone(),
                )
                .await?
            }
            McpTransport::HttpSse => {
                let url = server
                    .url
                    .as_ref()
                    .ok_or_else(|| "Missing url".to_string())?;
                McpClient::connect_http_sse(url, server.headers.clone()).await?
            }
        };

        let conn = McpConnection {
            client,
            tools_cache: Vec::new(),
            resources_cache: Vec::new(),
            last_error: None,
        };
        let mut conns = self.connections.lock().await;
        conns.insert(id.to_string(), conn);
        Ok(())
    }

    pub async fn disconnect(&self, id: &str) -> Result<(), String> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.remove(id) {
            conn.client.shutdown().await;
        }
        Ok(())
    }

    pub async fn status(&self, id: Option<String>) -> Vec<McpServerStatus> {
        let cfg = self.config.lock().await;
        let conns = self.connections.lock().await;
        let ids: Vec<String> = match id {
            Some(id) => vec![id],
            None => cfg.servers.iter().map(|s| s.id.clone()).collect(),
        };

        ids.into_iter()
            .map(|id| {
                if let Some(conn) = conns.get(&id) {
                    McpServerStatus {
                        id,
                        connected: true,
                        last_error: conn.last_error.clone(),
                        tools_cached: conn.tools_cache.len(),
                        resources_cached: conn.resources_cache.len(),
                    }
                } else {
                    McpServerStatus {
                        id,
                        connected: false,
                        last_error: None,
                        tools_cached: 0,
                        resources_cached: 0,
                    }
                }
            })
            .collect()
    }

    pub async fn tools_list(&self, id: &str) -> Result<Vec<ToolDefinition>, String> {
        let allowlist = self
            .get_server_allowlist(id, true)
            .await?
            .unwrap_or_default();
        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;
        let result = conn.client.request("tools/list", None).await?;

        let tools = extract_list(&result, "tools");
        let filtered = apply_allowlist_by_field(tools, &allowlist, "name");
        conn.tools_cache = filtered.clone();
        Ok(filtered)
    }

    pub async fn tools_call(
        &self,
        id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let allowlist = self
            .get_server_allowlist(id, true)
            .await?
            .unwrap_or_default();
        if !allowlist.is_empty() && !allowlist.contains(&tool_name.to_string()) {
            return Err("Tool not allowed".to_string());
        }

        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;

        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });
        conn.client.request("tools/call", Some(params)).await
    }

    pub async fn resources_list(&self, id: &str) -> Result<Vec<ResourceDefinition>, String> {
        let allowlist = self
            .get_server_allowlist(id, false)
            .await?
            .unwrap_or_default();
        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;
        let result = conn.client.request("resources/list", None).await?;

        let resources = extract_list(&result, "resources");
        let filtered = apply_allowlist_by_field(resources, &allowlist, "uri");
        conn.resources_cache = filtered.clone();
        Ok(filtered)
    }

    pub async fn resources_read(
        &self,
        id: &str,
        uri: &str,
    ) -> Result<serde_json::Value, String> {
        let allowlist = self
            .get_server_allowlist(id, false)
            .await?
            .unwrap_or_default();
        if !allowlist.is_empty() && !allowlist.contains(&uri.to_string()) {
            return Err("Resource not allowed".to_string());
        }

        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;

        let params = serde_json::json!({ "uri": uri });
        conn.client.request("resources/read", Some(params)).await
    }

    async fn get_server_allowlist(
        &self,
        id: &str,
        tools: bool,
    ) -> Result<Option<Vec<String>>, String> {
        let cfg = self.config.lock().await;
        let server = cfg
            .servers
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| "Server not found".to_string())?;
        if tools {
            Ok(server.tool_allowlist.clone())
        } else {
            Ok(server.resource_allowlist.clone())
        }
    }
}

pub struct McpConnection {
    client: McpClient,
    tools_cache: Vec<ToolDefinition>,
    resources_cache: Vec<ResourceDefinition>,
    last_error: Option<String>,
}

pub fn extract_list(result: &serde_json::Value, key: &str) -> Vec<serde_json::Value> {
    if let Some(list) = result.get(key).and_then(|v| v.as_array()) {
        return list.clone();
    }
    if let Some(list) = result.as_array() {
        return list.clone();
    }
    Vec::new()
}

pub fn apply_allowlist_by_field(
    items: Vec<serde_json::Value>,
    allowlist: &[String],
    field: &str,
) -> Vec<serde_json::Value> {
    if allowlist.is_empty() {
        return items;
    }
    items
        .into_iter()
        .filter(|item| {
            item.get(field)
                .and_then(|v| v.as_str())
                .map(|name| allowlist.contains(&name.to_string()))
                .unwrap_or(false)
        })
        .collect()
}

