use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use reqwest::header::{HeaderName, HeaderValue};
use rmcp::{
    model::{
        CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation,
        ReadResourceRequestParams,
    },
    transport::{
        child_process::TokioChildProcess,
        streamable_http_client::{
            StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
        },
    },
    ServiceExt,
};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::models::{
    McpCapabilities, McpConfig, McpInferredTool, McpServerConfig, McpServerStatus, McpTransport,
    ResourceDefinition, ToolDefinition,
};

#[derive(Clone)]
pub struct McpService {
    config: Arc<Mutex<McpConfig>>,
    connections: Arc<Mutex<HashMap<String, McpConnection>>>,
    capabilities: Arc<Mutex<HashMap<String, McpCapabilities>>>,
    stdio_connector: StdioConnector,
    resource_dir: Option<PathBuf>,
}

type StdioConnectFuture = Pin<Box<dyn Future<Output = Result<McpClient, String>> + Send>>;
type StdioConnector = Arc<
    dyn Fn(
            String,
            Vec<String>,
            Option<String>,
            Option<HashMap<String, String>>,
            Option<PathBuf>,
        ) -> StdioConnectFuture
        + Send
        + Sync,
>;

type McpRunningClient = rmcp::service::RunningService<rmcp::service::RoleClient, ClientInfo>;

#[derive(Clone)]
pub struct McpClient {
    inner: Arc<Mutex<McpRunningClient>>,
}

impl McpClient {
    async fn connect_stdio(
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: Option<HashMap<String, String>>,
        resource_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        let cmd = build_stdio_command(
            &command,
            &args,
            cwd.as_ref(),
            env.as_ref(),
            resource_dir.as_deref(),
        )
        .ok_or_else(|| format!("Failed to start MCP process: program not found: {command}"))?;
        #[cfg(windows)]
        let mut cmd = cmd;
        #[cfg(windows)]
        {
            cmd.creation_flags(0x08000000);
        }
        let transport =
            TokioChildProcess::new(cmd).map_err(|e| format!("Failed to start MCP process: {e}"))?;

        let client_info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("llama-desktop", env!("CARGO_PKG_VERSION")),
        );
        let client = client_info
            .serve(transport)
            .await
            .map_err(|e| format!("MCP stdio connect failed: {e}"))?;
        Ok(McpClient {
            inner: Arc::new(Mutex::new(client)),
        })
    }

    async fn connect_http_sse(
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self, String> {
        let mut header_map = HashMap::new();
        if let Some(headers) = headers {
            for (key, value) in headers {
                let name = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| format!("Invalid header name: {e}"))?;
                let value = HeaderValue::from_str(&value)
                    .map_err(|e| format!("Invalid header value: {e}"))?;
                header_map.insert(name, value);
            }
        }

        let http_client = reqwest::Client::new();
        let config = StreamableHttpClientTransportConfig::with_uri(url).custom_headers(header_map);
        let transport = StreamableHttpClientTransport::with_client(http_client, config);

        let client_info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("llama-desktop", env!("CARGO_PKG_VERSION")),
        );
        let client = client_info
            .serve(transport)
            .await
            .map_err(|e| format!("MCP http connect failed: {e}"))?;
        Ok(McpClient {
            inner: Arc::new(Mutex::new(client)),
        })
    }

    async fn shutdown(&self) {
        let mut client = self.inner.lock().await;
        let _ = client.close().await;
    }

    async fn list_all_tools(&self) -> Result<Vec<ToolDefinition>, String> {
        let client = self.inner.lock().await;
        let tools = client.list_all_tools().await.map_err(|e| e.to_string())?;

        let values = tools
            .into_iter()
            .filter_map(|tool| serde_json::to_value(tool).ok())
            .collect();
        Ok(values)
    }

    async fn list_all_resources(&self) -> Result<Vec<ResourceDefinition>, String> {
        let client = self.inner.lock().await;
        let resources = client
            .list_all_resources()
            .await
            .map_err(|e| e.to_string())?;

        let values = resources
            .into_iter()
            .filter_map(|resource| serde_json::to_value(resource).ok())
            .collect();
        Ok(values)
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let args = match arguments {
            serde_json::Value::Object(map) => map,
            serde_json::Value::Null => serde_json::Map::new(),
            other => {
                let mut map = serde_json::Map::new();
                map.insert("value".to_string(), other);
                map
            }
        };
        let request = CallToolRequestParams::new(tool_name.to_string()).with_arguments(args);
        let client = self.inner.lock().await;
        let result = client.call_tool(request).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }

    async fn read_resource(&self, uri: &str) -> Result<serde_json::Value, String> {
        let request = ReadResourceRequestParams::new(uri);
        let client = self.inner.lock().await;
        let result = client
            .read_resource(request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

fn build_stdio_command(
    command: &str,
    args: &[String],
    cwd: Option<&String>,
    env: Option<&HashMap<String, String>>,
    resource_dir: Option<&Path>,
) -> Option<Command> {
    let resolved = resolve_command(command, env, resource_dir);
    let use_shell = resolved.is_none();

    let mut cmd = if use_shell {
        let full = build_shell_command(command, args);
        if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", &full]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", &full]);
            cmd
        }
    } else {
        let mut cmd = Command::new(resolved?);
        cmd.args(args);
        cmd
    };

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env) = env {
        cmd.envs(env.clone());
    }

    Some(cmd)
}

fn build_shell_command(command: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_escape(command));
    for arg in args {
        parts.push(shell_escape(arg));
    }
    parts.join(" ")
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quotes = value.chars().any(|c| c.is_whitespace() || c == '"');
    if !needs_quotes {
        return value.to_string();
    }
    let escaped = value.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

fn resolve_command(
    command: &str,
    env_map: Option<&HashMap<String, String>>,
    resource_dir: Option<&Path>,
) -> Option<String> {
    let get_env = |key: &str| {
        env_map
            .and_then(|env| env.get(key).cloned())
            .or_else(|| env::var(key).ok())
    };

    let command_path = Path::new(command);
    if command_path.components().count() > 1 {
        if command_path.exists() {
            if cfg!(windows) && command_path.extension().is_none() {
                let pathext =
                    get_env("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
                for ext in pathext.split(';') {
                    if ext.trim().is_empty() {
                        continue;
                    }
                    let candidate = command_path.with_extension(ext.trim().trim_start_matches('.'));
                    if candidate.exists() {
                        return Some(candidate.to_string_lossy().to_string());
                    }
                }
                return None;
            }
            return Some(command.to_string());
        }
        if cfg!(windows) && command_path.extension().is_none() {
            let pathext = get_env("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
            for ext in pathext.split(';') {
                if ext.trim().is_empty() {
                    continue;
                }
                let candidate = command_path.with_extension(ext.trim().trim_start_matches('.'));
                if candidate.exists() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }
        return None;
    }

    let path_value = get_env("PATH").unwrap_or_default();

    let mut candidates = vec![command.to_string()];
    if cfg!(windows) {
        let pathext = get_env("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
        let exts: Vec<&str> = pathext.split(';').collect();
        candidates = exts
            .into_iter()
            .map(|ext| format!("{command}{ext}"))
            .collect();
        candidates.insert(0, command.to_string());
    }

    for dir in path_value.split(';') {
        if dir.trim().is_empty() {
            continue;
        }
        for candidate in &candidates {
            let path = PathBuf::from(dir).join(candidate);
            if !path.exists() {
                continue;
            }
            if cfg!(windows) {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let upper = ext.to_ascii_uppercase();
                    if upper == "EXE" || upper == "CMD" || upper == "BAT" || upper == "COM" {
                        return Some(path.to_string_lossy().to_string());
                    }
                    continue;
                }
                continue;
            }
            return Some(path.to_string_lossy().to_string());
        }
    }

    if let Some(resource_dir) = resource_dir {
        if let Some(path) = resolve_embedded_command(resource_dir, command) {
            return Some(path);
        }
    }

    if cfg!(windows) && command.eq_ignore_ascii_case("npx") {
        if let Some(app_data) = get_env("APPDATA") {
            let app_path = PathBuf::from(app_data).join("npm").join("npx.cmd");
            if app_path.exists() {
                return Some(app_path.to_string_lossy().to_string());
            }
        }
        if let Some(program_files) = get_env("PROGRAMFILES") {
            let pf_path = PathBuf::from(program_files).join("nodejs").join("npx.cmd");
            if pf_path.exists() {
                return Some(pf_path.to_string_lossy().to_string());
            }
        }
        if let Some(program_files) = get_env("PROGRAMFILES(X86)") {
            let pfx_path = PathBuf::from(program_files).join("nodejs").join("npx.cmd");
            if pfx_path.exists() {
                return Some(pfx_path.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn resolve_embedded_command(resource_dir: &Path, command: &str) -> Option<String> {
    let root = resource_dir.join("node").join("bin");
    if command.eq_ignore_ascii_case("npx") {
        let candidates = if cfg!(target_os = "windows") {
            vec![root.join("npx.cmd")]
        } else {
            vec![root.join("npx")]
        };
        for candidate in candidates {
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }

    if command.eq_ignore_ascii_case("node") {
        let candidates = if cfg!(target_os = "windows") {
            vec![root.join("node.exe")]
        } else {
            vec![root.join("node")]
        };
        for candidate in candidates {
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }

    None
}

impl McpService {
    pub fn new(config: McpConfig, resource_dir: Option<PathBuf>) -> Self {
        Self::new_with_stdio_connector(config, resource_dir, |command, args, cwd, env, dir| {
            Box::pin(McpClient::connect_stdio(command, args, cwd, env, dir))
        })
    }

    pub fn new_with_stdio_connector<F, Fut>(
        config: McpConfig,
        resource_dir: Option<PathBuf>,
        connector: F,
    ) -> Self
    where
        F: Fn(
                String,
                Vec<String>,
                Option<String>,
                Option<HashMap<String, String>>,
                Option<PathBuf>,
            ) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Result<McpClient, String>> + Send + 'static,
    {
        let stdio_connector: StdioConnector = Arc::new(move |command, args, cwd, env, dir| {
            Box::pin(connector(command, args, cwd, env, dir))
        });
        Self {
            config: Arc::new(Mutex::new(config)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            capabilities: Arc::new(Mutex::new(HashMap::new())),
            stdio_connector,
            resource_dir,
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
        let found = cfg.servers.iter().any(|s| s.id == id);
        if !found {
            let conns = self.connections.lock().await;
            if !conns.contains_key(id) {
                return Err("Server not found".to_string());
            }
        }

        cfg.servers.retain(|s| s.id != id);
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.remove(id) {
            conn.client.shutdown().await;
        }
        let mut caps_map = self.capabilities.lock().await;
        caps_map.remove(id);
        Ok(())
    }

    pub async fn connect(&self, id: &str) -> Result<(), String> {
        // If already connected, skip — avoids killing stdio subprocesses on every tool call.
        {
            let conns = self.connections.lock().await;
            if conns.contains_key(id) {
                return Ok(());
            }
        }

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
                    self.resource_dir.clone(),
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
        drop(conns);

        if let Ok(caps) = self.detect_capabilities(id).await {
            let mut caps_map = self.capabilities.lock().await;
            caps_map.insert(id.to_string(), caps);
        }
        Ok(())
    }

    async fn detect_capabilities(&self, id: &str) -> Result<McpCapabilities, String> {
        let client = {
            let conns = self.connections.lock().await;
            let conn = conns
                .get(id)
                .ok_or_else(|| "Server not connected".to_string())?;
            conn.client.clone()
        };

        let mut caps = McpCapabilities::default();

        match client.list_all_tools().await {
            Ok(tools) => {
                caps.has_tools_list = true;
                caps.supports_call_tools = true;
                caps.inferred_tools = extract_inferred_tools(&tools);
            }
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    caps.has_tools_list = false;
                    caps.supports_call_tools = false;
                    caps.last_error = Some(err);
                }
                McpCallError::Transport(err) => {
                    caps.last_error = Some(err);
                }
            },
        }

        match client.list_all_resources().await {
            Ok(_) => {
                caps.has_resources_list = true;
                caps.supports_resources_read = true;
            }
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    caps.has_resources_list = false;
                    caps.supports_resources_read = false;
                    if caps.last_error.is_none() {
                        caps.last_error = Some(err);
                    }
                }
                McpCallError::Transport(err) => {
                    if caps.last_error.is_none() {
                        caps.last_error = Some(err);
                    }
                }
            },
        }

        Ok(caps)
    }

    pub async fn disconnect(&self, id: &str) -> Result<(), String> {
        let mut conns = self.connections.lock().await;
        let Some(conn) = conns.remove(id) else {
            return Err("Server not connected".to_string());
        };
        conn.client.shutdown().await;
        let mut caps_map = self.capabilities.lock().await;
        caps_map.remove(id);
        Ok(())
    }

    pub async fn status(&self, id: Option<String>) -> Vec<McpServerStatus> {
        let cfg = self.config.lock().await;
        let conns = self.connections.lock().await;
        let caps_map = self.capabilities.lock().await;
        let ids: Vec<String> = match id {
            Some(id) => vec![id],
            None => cfg.servers.iter().map(|s| s.id.clone()).collect(),
        };

        ids.into_iter()
            .map(|id| {
                let caps = caps_map.get(&id).cloned();
                if let Some(conn) = conns.get(&id) {
                    McpServerStatus {
                        id,
                        connected: true,
                        last_error: conn.last_error.clone(),
                        tools_cached: conn.tools_cache.len(),
                        resources_cached: conn.resources_cache.len(),
                        capabilities: caps,
                    }
                } else {
                    McpServerStatus {
                        id,
                        connected: false,
                        last_error: None,
                        tools_cached: 0,
                        resources_cached: 0,
                        capabilities: caps,
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
        let result = conn.client.list_all_tools().await;
        let tools = match result {
            Ok(tools) => tools,
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    conn.last_error = Some("Server does not support tools/list".to_string());
                    conn.tools_cache = Vec::new();
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.has_tools_list = false;
                    caps.supports_call_tools = false;
                    caps.inferred_tools = Vec::new();
                    caps.last_error = Some(err);
                    return Ok(Vec::new());
                }
                McpCallError::Transport(err) => {
                    conn.last_error = Some(err.clone());
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.last_error = Some(err.clone());
                    return Err(err);
                }
            },
        };

        let filtered = apply_allowlist_by_field(&tools, &allowlist, "name");
        conn.tools_cache = filtered.clone();
        conn.last_error = None;
        let mut caps_map = self.capabilities.lock().await;
        let caps = caps_map.entry(id.to_string()).or_default();
        caps.has_tools_list = true;
        caps.supports_call_tools = true;
        caps.inferred_tools = extract_inferred_tools(&tools);
        caps.last_error = None;
        Ok(filtered)
    }

    pub async fn call_tools(
        &self,
        id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let allowlist = self
            .get_server_allowlist(id, true)
            .await?
            .unwrap_or_default();
        if !allowlist_allows(&allowlist) && !allowlist.contains(&tool_name.to_string()) {
            return Err("Tool not allowed".to_string());
        }

        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;

        match conn.client.call_tool(tool_name, arguments).await {
            Ok(value) => Ok(value),
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.supports_call_tools = false;
                    caps.last_error = Some(err);
                    Err("Server does not support tools/call".to_string())
                }
                McpCallError::Transport(err) => {
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.last_error = Some(err.clone());
                    Err(err)
                }
            },
        }
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
        let result = conn.client.list_all_resources().await;
        let resources = match result {
            Ok(resources) => resources,
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    conn.last_error = Some("Server does not support resources/list".to_string());
                    conn.resources_cache = Vec::new();
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.has_resources_list = false;
                    caps.supports_resources_read = false;
                    caps.last_error = Some(err);
                    return Ok(Vec::new());
                }
                McpCallError::Transport(err) => {
                    conn.last_error = Some(err.clone());
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.last_error = Some(err.clone());
                    return Err(err);
                }
            },
        };

        let filtered = apply_allowlist_by_field(&resources, &allowlist, "uri");
        conn.resources_cache = filtered.clone();
        conn.last_error = None;
        let mut caps_map = self.capabilities.lock().await;
        let caps = caps_map.entry(id.to_string()).or_default();
        caps.has_resources_list = true;
        caps.supports_resources_read = true;
        caps.last_error = None;
        Ok(filtered)
    }

    pub async fn resources_read(&self, id: &str, uri: &str) -> Result<serde_json::Value, String> {
        let allowlist = self
            .get_server_allowlist(id, false)
            .await?
            .unwrap_or_default();
        if !allowlist_allows(&allowlist) && !allowlist.contains(&uri.to_string()) {
            return Err("Resource not allowed".to_string());
        }

        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(id)
            .ok_or_else(|| "Server not connected".to_string())?;

        match conn.client.read_resource(uri).await {
            Ok(value) => Ok(value),
            Err(err) => match classify_call_error(err) {
                McpCallError::Unsupported(err) => {
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.supports_resources_read = false;
                    caps.last_error = Some(err);
                    Err("Server does not support resources/read".to_string())
                }
                McpCallError::Transport(err) => {
                    let mut caps_map = self.capabilities.lock().await;
                    let caps = caps_map.entry(id.to_string()).or_default();
                    caps.last_error = Some(err.clone());
                    Err(err)
                }
            },
        }
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

fn is_method_not_found_error(message: &str) -> bool {
    message.contains("MCP error -32601") || message.contains("Method not found")
}

enum McpCallError {
    Unsupported(String),
    Transport(String),
}

fn classify_call_error(err: String) -> McpCallError {
    if is_method_not_found_error(&err) {
        McpCallError::Unsupported(err)
    } else {
        McpCallError::Transport(err)
    }
}

fn extract_inferred_tools(tools: &[ToolDefinition]) -> Vec<McpInferredTool> {
    tools
        .iter()
        .filter_map(|item| {
            item.get("name")
                .and_then(|v| v.as_str())
                .map(|name| McpInferredTool {
                    name: name.to_string(),
                    method: "tools/call".to_string(),
                })
        })
        .collect()
}

pub fn apply_allowlist_by_field(
    items: &[serde_json::Value],
    allowlist: &[String],
    field: &str,
) -> Vec<serde_json::Value> {
    if allowlist_allows(allowlist) {
        return items.to_vec();
    }
    items
        .iter()
        .filter(|item| {
            item.get(field)
                .and_then(|v| v.as_str())
                .map(|name| allowlist.contains(&name.to_string()))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn allowlist_allows(allowlist: &[String]) -> bool {
    allowlist.is_empty() || allowlist.iter().any(|item| item == "*")
}
