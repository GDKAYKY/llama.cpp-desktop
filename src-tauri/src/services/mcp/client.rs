use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use futures::StreamExt;

use crate::services::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};

#[derive(Clone)]
pub enum McpClient {
    Stdio(Arc<Mutex<StdioClient>>),
    HttpSse(Arc<HttpClient>),
}

impl McpClient {
    pub async fn connect_stdio(
        command: &str,
        args: &[String],
        cwd: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        #[cfg(windows)]
        {
            // Avoid flashing a console window for stdio-based MCP servers.
            cmd.creation_flags(0x08000000);
        }
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        if let Some(env) = env {
            cmd.envs(env);
        }

        let mut child = cmd.spawn().map_err(|e| format!("Spawn failed: {}", e))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to open stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to open stdout".to_string())?;

        let client = StdioClient::new(Some(child), stdin, stdout);

        Ok(McpClient::Stdio(Arc::new(Mutex::new(client))))
    }

    pub async fn connect_stdio_owned(
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> Result<Self, String> {
        Self::connect_stdio(&command, &args, cwd, env).await
    }

    pub fn connect_stdio_with_io(
        stdin: impl AsyncWrite + Unpin + Send + 'static,
        stdout: impl AsyncRead + Unpin + Send + 'static,
    ) -> Self {
        let client = StdioClient::new(None, stdin, stdout);
        McpClient::Stdio(Arc::new(Mutex::new(client)))
    }

    pub async fn connect_http_sse(
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self, String> {
        let client = HttpClient::new(url.to_string(), headers)?;
        Ok(McpClient::HttpSse(Arc::new(client)))
    }

    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        match self {
            McpClient::Stdio(client) => {
                let mut client = client.lock().await;
                client.request(method, params).await
            }
            McpClient::HttpSse(client) => client.request(method, params).await,
        }
    }

    pub async fn shutdown(&self) {
        if let McpClient::Stdio(client) = self {
            let mut client = client.lock().await;
            client.shutdown().await;
        }
    }
}

pub struct StdioClient {
    child: Option<Child>,
    stdin: Box<dyn AsyncWrite + Unpin + Send>,
    stdout: BufReader<Box<dyn AsyncRead + Unpin + Send>>,
    next_id: u64,
}

impl StdioClient {
    fn new(
        child: Option<Child>,
        stdin: impl AsyncWrite + Unpin + Send + 'static,
        stdout: impl AsyncRead + Unpin + Send + 'static,
    ) -> Self {
        Self {
            child,
            stdin: Box::new(stdin),
            stdout: BufReader::new(Box::new(stdout)),
            next_id: 1,
        }
    }

    async fn request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let payload = serde_json::to_string(&req)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        self.stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| format!("Failed to write request: {}", e))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to write newline: {}", e))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush request: {}", e))?;

        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;
            if bytes == 0 {
                return Err("MCP server closed stdout".to_string());
            }
            let parsed: JsonRpcResponse = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if parsed.id == serde_json::Value::from(id) {
                if let Some(err) = parsed.error {
                    return Err(format!("MCP error {}: {}", err.code, err.message));
                }
                return parsed.result.ok_or_else(|| "Missing result".to_string());
            }
        }
    }

    async fn shutdown(&mut self) {
        // Close stdin for in-memory transports so the server task can exit.
        let _ = self.stdin.shutdown().await;
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
}

pub struct HttpClient {
    url: String,
    headers: HashMap<String, String>,
    client: reqwest::Client,
    next_id: Arc<Mutex<u64>>,
}

impl HttpClient {
    pub fn new(url: String, headers: Option<HashMap<String, String>>) -> Result<Self, String> {
        Ok(Self {
            url,
            headers: headers.unwrap_or_default(),
            client: reqwest::Client::new(),
            next_id: Arc::new(Mutex::new(1)),
        })
    }

    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let mut builder = self
            .client
            .post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .json(&req);
        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        if !response.status().is_success() {
            return Err(format!("HTTP request failed: {}", response.status()));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        if content_type.contains("text/event-stream") {
            return self.read_sse_response(response, id).await;
        }

        let parsed: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(err) = parsed.error {
            return Err(format!("MCP error {}: {}", err.code, err.message));
        }
        parsed.result.ok_or_else(|| "Missing result".to_string())
    }

    async fn read_sse_response(
        &self,
        response: reqwest::Response,
        id: u64,
    ) -> Result<serde_json::Value, String> {
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut current_event = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Failed to read SSE chunk: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some((line, rest)) = split_next_line(&buffer) {
                buffer = rest;
                let trimmed = line.trim_end_matches('\r');

                if trimmed.is_empty() {
                    if let Some(data) = extract_sse_data(&current_event) {
                        if let Some(result) = parse_jsonrpc_response_data(&data, id)? {
                            return Ok(result);
                        }
                    } else if let Some(result) = try_parse_jsonrpc_inline(&current_event, id)? {
                        return Ok(result);
                    }
                    current_event.clear();
                    continue;
                }

                current_event.push_str(trimmed);
                current_event.push('\n');
            }
        }

        if let Some(data) = extract_sse_data(&current_event) {
            if let Some(result) = parse_jsonrpc_response_data(&data, id)? {
                return Ok(result);
            }
        } else if let Some(result) = try_parse_jsonrpc_inline(&current_event, id)? {
            return Ok(result);
        }

        Err("SSE stream ended without response".to_string())
    }
}

fn extract_sse_data(raw_event: &str) -> Option<String> {
    let mut data_lines = Vec::new();
    for line in raw_event.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start());
        }
    }
    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}

fn parse_jsonrpc_response_data(
    data: &str,
    id: u64,
) -> Result<Option<serde_json::Value>, String> {
    if data.trim().is_empty() {
        return Ok(None);
    }

    let value: serde_json::Value =
        serde_json::from_str(data).map_err(|e| format!("Failed to parse SSE data: {}", e))?;

    let mut responses = Vec::new();
    match value {
        serde_json::Value::Array(items) => responses.extend(items),
        other => responses.push(other),
    }

    for item in responses {
        let parsed: JsonRpcResponse = match serde_json::from_value(item) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if id_matches(&parsed.id, id) {
            if let Some(err) = parsed.error {
                return Err(format!("MCP error {}: {}", err.code, err.message));
            }
            return Ok(parsed.result);
        }
    }

    Ok(None)
}

fn split_next_line(input: &str) -> Option<(String, String)> {
    let mut chars = input.char_indices();
    for (idx, ch) in &mut chars {
        if ch == '\n' {
            let line = input[..idx].to_string();
            let rest = input[idx + 1..].to_string();
            return Some((line, rest));
        }
    }
    None
}

fn try_parse_jsonrpc_inline(
    raw_event: &str,
    id: u64,
) -> Result<Option<serde_json::Value>, String> {
    let trimmed = raw_event.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let value: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    let mut responses = Vec::new();
    match value {
        serde_json::Value::Array(items) => responses.extend(items),
        other => responses.push(other),
    }

    for item in responses {
        let parsed: JsonRpcResponse = match serde_json::from_value(item) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if id_matches(&parsed.id, id) {
            if let Some(err) = parsed.error {
                return Err(format!("MCP error {}: {}", err.code, err.message));
            }
            return Ok(parsed.result);
        }
    }

    Ok(None)
}

fn id_matches(value: &serde_json::Value, id: u64) -> bool {
    match value {
        serde_json::Value::Number(num) => num.as_u64() == Some(id),
        serde_json::Value::String(s) => s == &id.to_string(),
        _ => false,
    }
}
