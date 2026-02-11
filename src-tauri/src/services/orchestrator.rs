use crate::models::ChatMessage;
use crate::services::llama::service::LlamaCppService;
use crate::services::mcp::McpService;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ChatOrchestrator {
    sessions: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    service: LlamaCppService,
    mcp_service: McpService,
}

impl ChatOrchestrator {
    pub fn new(service: LlamaCppService, mcp_service: McpService) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            service,
            mcp_service,
        }
    }

    pub async fn process(
        &self,
        session_id: &str,
        user_input: String,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let (mcp_ids, cleaned_input) = extract_mcp_ids(&user_input);

        let mut sessions = self.sessions.lock().await;
        let history = sessions
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);

        history.push(ChatMessage {
            role: "user".to_string(),
            content: cleaned_input.clone(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });

        let mut messages = history.clone();
        drop(sessions);

        if !mcp_ids.is_empty() {
            self.run_with_tools(
                session_id,
                &mcp_ids,
                messages,
                temperature,
                max_tokens,
                on_event,
            )
            .await
        } else {
            self.run_streaming(session_id, messages, temperature, max_tokens, on_event)
                .await
        }
    }

    async fn run_streaming(
        &self,
        session_id: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let mut rx = self
            .service
            .send_chat_message(
                Some(session_id.to_string()),
                messages,
                temperature,
                0.95,
                40,
                max_tokens,
            )
            .await?;

        let mut full_response = String::new();

        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
            let _ = on_event.send(serde_json::json!({
                "chunk": chunk
            }));
        }

        let _ = on_event.send(serde_json::json!({
            "status": "done"
        }));

        let mut sessions = self.sessions.lock().await;
        if let Some(history) = sessions.get_mut(session_id) {
            history.push(ChatMessage {
                role: "assistant".to_string(),
                content: full_response,
                name: None,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        Ok(())
    }

    async fn run_with_tools(
        &self,
        session_id: &str,
        mcp_ids: &[String],
        mut messages: Vec<ChatMessage>,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let mut listed_once = false;
        let mut browsed_servers = HashSet::new();
        ensure_tool_system_prompt(&mut messages, mcp_ids);

        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > 8 {
                return Err("Tool loop exceeded limit".to_string());
            }

            let response = self
                .service
                .complete_chat(
                    Some(session_id.to_string()),
                    messages.clone(),
                    temperature,
                    0.95,
                    40,
                    max_tokens,
                    None,
                    None,
                )
                .await?;

            let assistant_message = parse_assistant_message(&response)
                .ok_or_else(|| "Model response is missing assistant message".to_string())?;
            let assistant_content = assistant_message.content.clone();
            messages.push(assistant_message);

            if let Some(action) = parse_action_json(&assistant_content) {
                let result = self
                    .execute_mcp_action(
                        &action,
                        mcp_ids,
                        &mut listed_once,
                        &mut browsed_servers,
                    )
                    .await?;
                messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "MCP_RESULT:\n{}\nContinue. If more external data is needed, emit the next JSON action only.",
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                    ),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                });
                continue;
            }

            let _ = on_event.send(serde_json::json!({
                "chunk": assistant_content
            }));
            let _ = on_event.send(serde_json::json!({
                "status": "done"
            }));

            let mut sessions = self.sessions.lock().await;
            if let Some(history) = sessions.get_mut(session_id) {
                *history = messages;
            }
            return Ok(());
        }
    }

    async fn execute_mcp_action(
        &self,
        action: &McpAction,
        allowed_servers: &[String],
        listed_once: &mut bool,
        browsed_servers: &mut HashSet<String>,
    ) -> Result<serde_json::Value, String> {
        match action.tool.as_str() {
            "mcp.list_servers" => {
                let mut servers = self.mcp_service.list_servers().await;
                if !allowed_servers.is_empty() {
                    servers.retain(|server| allowed_servers.contains(&server.id));
                }
                *listed_once = true;
                Ok(serde_json::json!({
                    "tool": "mcp.list_servers",
                    "servers": servers
                        .into_iter()
                        .map(|server| serde_json::json!({
                            "id": server.id,
                            "name": server.name,
                            "enabled": server.enabled,
                            "transport": server.transport,
                        }))
                        .collect::<Vec<_>>()
                }))
            }
            "mcp.browse_resources" => {
                if !*listed_once {
                    return Ok(serde_json::json!({
                        "error": "Call mcp.list_servers first."
                    }));
                }
                let server = action
                    .server
                    .as_ref()
                    .ok_or_else(|| "mcp.browse_resources requires 'server'".to_string())?;
                if !allowed_servers.is_empty() && !allowed_servers.contains(server) {
                    return Ok(serde_json::json!({
                        "error": format!("Server '{}' is not allowed for this request.", server)
                    }));
                }
                self.mcp_service.connect(server).await?;
                let resources = self.mcp_service.resources_list(server).await?;
                browsed_servers.insert(server.clone());
                Ok(serde_json::json!({
                    "tool": "mcp.browse_resources",
                    "server": server,
                    "resources": resources
                }))
            }
            "mcp.run" => {
                let server = action
                    .server
                    .as_ref()
                    .ok_or_else(|| "mcp.run requires 'server'".to_string())?;
                if !*listed_once {
                    return Ok(serde_json::json!({
                        "error": "Call mcp.list_servers before mcp.run."
                    }));
                }
                if !browsed_servers.contains(server) {
                    return Ok(serde_json::json!({
                        "error": format!("Call mcp.browse_resources for '{}' before mcp.run.", server)
                    }));
                }
                if !allowed_servers.is_empty() && !allowed_servers.contains(server) {
                    return Ok(serde_json::json!({
                        "error": format!("Server '{}' is not allowed for this request.", server)
                    }));
                }
                let resource = action
                    .resource
                    .as_ref()
                    .ok_or_else(|| "mcp.run requires 'resource'".to_string())?;
                self.mcp_service.connect(server).await?;
                let args = match action.input.clone() {
                    Some(value) if value.is_object() => value,
                    Some(value) => serde_json::json!({ "input": value }),
                    None => serde_json::json!({}),
                };
                let result = self.mcp_service.tools_call(server, resource, args).await?;
                Ok(serde_json::json!({
                    "tool": "mcp.run",
                    "server": server,
                    "resource": resource,
                    "result": result
                }))
            }
            _ => Ok(serde_json::json!({
                "error": format!("Unknown action '{}'. Allowed: mcp.list_servers, mcp.browse_resources, mcp.run", action.tool)
            })),
        }
    }

    pub async fn clear_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(session_id);
    }

    pub async fn get_message(&self, session_id: &str, message_index: usize) -> Option<ChatMessage> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id)
            .and_then(|history| history.get(message_index).cloned())
    }

    pub async fn set_session_history(&self, session_id: &str, history: Vec<ChatMessage>) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.to_string(), history);
    }

    pub fn prepare_regenerate_history(
        history: &[ChatMessage],
        message_index: usize,
    ) -> Result<Vec<ChatMessage>, String> {
        let target = history
            .get(message_index)
            .ok_or_else(|| "Message not found".to_string())?;

        if target.role != "assistant" {
            return Err("Target message is not an assistant response".to_string());
        }

        Ok(history[..message_index].to_vec())
    }

    pub async fn regenerate_at(
        &self,
        session_id: &str,
        message_index: usize,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let history_before = {
            let sessions = self.sessions.lock().await;
            let history = sessions
                .get(session_id)
                .ok_or_else(|| "Session not found".to_string())?;
            Self::prepare_regenerate_history(history, message_index)?
        };

        let mut rx = self
            .service
            .send_chat_message(
                Some(session_id.to_string()),
                history_before,
                temperature,
                0.95,
                40,
                max_tokens,
            )
            .await?;

        let mut full_response = String::new();

        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
            let _ = on_event.send(serde_json::json!({
                "chunk": chunk
            }));
        }

        let _ = on_event.send(serde_json::json!({
            "status": "done"
        }));

        let mut sessions = self.sessions.lock().await;
        let history = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if message_index >= history.len() {
            return Err("Message no longer exists".to_string());
        }

        history[message_index].content = full_response;

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct McpAction {
    tool: String,
    server: Option<String>,
    resource: Option<String>,
    input: Option<serde_json::Value>,
}

fn extract_mcp_ids(input: &str) -> (Vec<String>, String) {
    let mut ids: Vec<String> = Vec::new();
    let mut cleaned_tokens: Vec<&str> = Vec::new();

    for token in input.split_whitespace() {
        if let Some(id) = parse_mcp_token(token) {
            if !ids.contains(&id) {
                ids.push(id);
            }
            continue;
        }
        cleaned_tokens.push(token);
    }

    (ids, cleaned_tokens.join(" ").trim().to_string())
}

fn parse_mcp_token(token: &str) -> Option<String> {
    for prefix in ["@mcp:", "/mcp:"] {
        if let Some(raw) = token.strip_prefix(prefix) {
            let id = raw
                .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

fn parse_assistant_message(response: &serde_json::Value) -> Option<ChatMessage> {
    let message = response
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("message"))
        .cloned();

    if let Some(message) = message {
        let content = message
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return Some(ChatMessage {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });
    }

    None
}

fn parse_action_json(content: &str) -> Option<McpAction> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(action) = serde_json::from_str::<McpAction>(trimmed) {
        return Some(action);
    }

    let fenced = extract_fenced_json(trimmed)?;
    serde_json::from_str::<McpAction>(&fenced).ok()
}

fn extract_fenced_json(content: &str) -> Option<String> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(content[start..=end].to_string())
}

fn ensure_tool_system_prompt(messages: &mut Vec<ChatMessage>, mcp_ids: &[String]) {
    if matches!(messages.first(), Some(msg) if msg.role == "system") {
        return;
    }
    let joined = mcp_ids.join(", ");
    messages.insert(
        0,
        ChatMessage {
            role: "system".to_string(),
            content: format!("You may request tools by responding ONLY with JSON.\nAllowed actions:\n- {{\"tool\":\"mcp.list_servers\"}}\n- {{\"tool\":\"mcp.browse_resources\",\"server\":\"<id>\"}}\n- {{\"tool\":\"mcp.run\",\"server\":\"<id>\",\"resource\":\"<name>\",\"input\":{{...}}}}\nRules:\n1) If external data is required, call mcp.list_servers first.\n2) Then call mcp.browse_resources before mcp.run.\n3) Never hallucinate MCP output.\nAllowed server ids for this request: [{}].\nWhen done, answer normally in plain text (not JSON).", joined),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    );
}
