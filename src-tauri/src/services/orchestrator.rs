use crate::models::ChatMessage;
use crate::services::capability_registry::{CapabilityRegistry, LlmToolSpecBundle, ResolvedCall};
use crate::services::llama::service::LlamaCppService;
use crate::services::mcp::McpService;
use crate::services::thinking_parser::{ParsedChunk, ThinkingStreamParser};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::Mutex;

const MAX_TOOL_ITERATIONS: usize = 5;
const TOOL_CALL_MAX_TOKENS: i32 = 1024;
const TOOL_RESULT_TOKEN_BUDGET: usize = 512;

#[derive(Clone)]
pub struct ChatOrchestrator {
    sessions: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    service: LlamaCppService,
    mcp_service: McpService,
    registry: CapabilityRegistry,
}

impl ChatOrchestrator {
    pub fn new(service: LlamaCppService, mcp_service: McpService) -> Self {
        let registry = CapabilityRegistry::new();

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            service,
            mcp_service,
            registry,
        }
    }

    fn try_send(on_event: &Channel<serde_json::Value>, payload: serde_json::Value) -> bool {
        on_event.send(payload).is_ok()
    }

    /// Call once at startup and whenever MCP config changes.
    pub async fn refresh_capabilities(&self) -> Result<(), String> {
        self.registry.refresh(&self.mcp_service).await
    }

    // ══════════════════════════════════════════════════════════════
    //  MAIN ENTRY POINT
    // ══════════════════════════════════════════════════════════════

    pub async fn process(
        &self,
        session_id: &str,
        user_input: String,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        // Guard against race condition: if registry is empty (startup refresh still running),
        // attempt a blocking refresh before processing.
        if self.registry.available_server_ids().await.is_empty() {
            let _ = self.refresh_capabilities().await;
        }

        let (mentioned_mcp_ids, cleaned_input) = extract_mcp_ids(&user_input);
        let allowed_servers = if !mentioned_mcp_ids.is_empty() {
            mentioned_mcp_ids
        } else {
            self.registry.available_server_ids().await
        };

        self.append_message(
            session_id,
            ChatMessage {
                role: "user".to_string(),
                content: cleaned_input.clone(),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
        )
        .await;

        if allowed_servers.is_empty() {
            let messages = self.get_history(session_id).await;
            return self
                .run_streaming(session_id, messages, temperature, max_tokens, on_event)
                .await;
        }

        let tool_bundle = self
            .registry
            .llm_tools_for_query(&cleaned_input, &allowed_servers, 0)
            .await;

        if tool_bundle.tools.is_empty() {
            let messages = self.get_history(session_id).await;
            return self
                .run_streaming(session_id, messages, temperature, max_tokens, on_event)
                .await;
        }

        let mut iteration = 0usize;
        let mut seen_calls: HashSet<String> = HashSet::new();

        loop {
            iteration += 1;
            if iteration > MAX_TOOL_ITERATIONS {
                if !Self::try_send(
                    &on_event,
                    serde_json::json!({
                        "thinking": format!(
                            "Tool loop exceeded max iterations ({}). Streaming final answer.",
                            MAX_TOOL_ITERATIONS
                        )
                    }),
                ) {
                    return Ok(());
                }
                let messages = self.get_history(session_id).await;
                return self
                    .run_streaming(session_id, messages, temperature, max_tokens, on_event)
                    .await;
            }

            let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
            let tool_max_tokens = clamp_max_tokens(ctx_size, TOOL_CALL_MAX_TOKENS);
            let prompt_budget = compute_prompt_budget(ctx_size, tool_max_tokens);
            let history = self.get_history(session_id).await;
            let request_messages =
                sanitize_messages_for_request(trim_messages_to_budget(&history, prompt_budget));

            if !Self::try_send(
                &on_event,
                serde_json::json!({
                    "thinking": format!("Tool loop iteration {}", iteration)
                }),
            ) {
                return Ok(());
            }

            let response = self
                .service
                .complete_chat(
                    None,
                    request_messages,
                    temperature.min(0.5),
                    0.95,
                    40,
                    tool_max_tokens,
                    Some(tool_bundle.tools.clone()),
                    None,
                )
                .await?;

            let parsed = parse_tool_calls_from_response(&response)?;
            if parsed.tool_calls.is_empty() {
                if !Self::try_send(
                    &on_event,
                    serde_json::json!({
                        "thinking": "No tool calls detected. Streaming final answer."
                    }),
                ) {
                    return Ok(());
                }
                let messages = self.get_history(session_id).await;
                return self
                    .run_streaming(session_id, messages, temperature, max_tokens, on_event)
                    .await;
            }

            self.append_message(
                session_id,
                ChatMessage {
                    role: "assistant".to_string(),
                    content: parsed.content,
                    name: None,
                    tool_call_id: None,
                    tool_calls: Some(parsed.raw_tool_calls),
                },
            )
            .await;

            let repeat_detected = self
                .execute_tool_calls(
                    session_id,
                    &cleaned_input,
                    &allowed_servers,
                    &tool_bundle,
                    &parsed.tool_calls,
                    &mut seen_calls,
                    &on_event,
                )
                .await?;

            if repeat_detected {
                if !Self::try_send(
                    &on_event,
                    serde_json::json!({
                        "thinking": "Repeated tool call detected. Streaming final answer."
                    }),
                ) {
                    return Ok(());
                }
                let messages = self.get_history(session_id).await;
                return self
                    .run_streaming(session_id, messages, temperature, max_tokens, on_event)
                    .await;
            }
        }
    }

    async fn append_message(&self, session_id: &str, message: ChatMessage) {
        let mut sessions = self.sessions.lock().await;
        let history = sessions
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);
        history.push(message);
    }

    async fn execute_tool_calls(
        &self,
        session_id: &str,
        original_query: &str,
        allowed_servers: &[String],
        tool_bundle: &LlmToolSpecBundle,
        tool_calls: &[LlmToolCall],
        seen_calls: &mut HashSet<String>,
        on_event: &Channel<serde_json::Value>,
    ) -> Result<bool, String> {
        let mut repeat_detected = false;

        for (idx, call) in tool_calls.iter().enumerate() {
            let fingerprint = format!(
                "{}:{}",
                call.tool_id,
                serde_json::to_string(&call.arguments).unwrap_or_default()
            );
            if !seen_calls.insert(fingerprint) {
                repeat_detected = true;
                self.append_tool_error(
                    session_id,
                    &call.id,
                    format!("Repeated tool call for '{}'", call.tool_id),
                )
                .await;
                continue;
            }

            let (server_id, tool_name) = match resolve_tool_id(&call.tool_id, tool_bundle) {
                Some(pair) => pair,
                None => {
                    self.append_tool_error(
                        session_id,
                        &call.id,
                        format!("Unknown tool id '{}'", call.tool_id),
                    )
                    .await;
                    continue;
                }
            };

            if !allowed_servers.contains(&server_id) {
                self.append_tool_error(
                    session_id,
                    &call.id,
                    format!("Server '{}' not allowed", server_id),
                )
                .await;
                continue;
            }

            let arguments = self
                .build_tool_arguments(
                    original_query,
                    &server_id,
                    &tool_name,
                    &call.arguments,
                    call.arguments_valid,
                )
                .await;

            let resolved = ResolvedCall {
                server_id: server_id.clone(),
                tool_name: tool_name.clone(),
                arguments: arguments.clone(),
            };

            if let Err(e) = self.registry.validate_call(&resolved).await {
                self.append_tool_error(session_id, &call.id, e).await;
                continue;
            }

            if !Self::try_send(
                &on_event,
                serde_json::json!({
                    "thinking": format!("Calling MCP tool {}::{}", server_id, tool_name)
                }),
            ) {
                return Ok(repeat_detected);
            }

            let result = match self.mcp_service.connect(&server_id).await {
                Ok(()) => {
                    self.mcp_service
                        .tools_call(&server_id, &tool_name, arguments)
                        .await
                }
                Err(e) => Err(e),
            };

            let content = match result {
                Ok(res) => format_tool_result(&res),
                Err(e) => {
                    if !Self::try_send(
                        &on_event,
                        serde_json::json!({
                            "thinking": format!("Tool call failed: {}", e)
                        }),
                    ) {
                        return Ok(repeat_detected);
                    }
                    serde_json::json!({ "error": e }).to_string()
                }
            };

            self.append_message(
                session_id,
                ChatMessage {
                    role: "tool".to_string(),
                    content,
                    name: None,
                    tool_call_id: Some(call.id.clone()),
                    tool_calls: None,
                },
            )
            .await;

            if idx + 1 == tool_calls.len() {
                if !Self::try_send(
                    &on_event,
                    serde_json::json!({
                        "thinking": "Tool results injected into context."
                    }),
                ) {
                    return Ok(repeat_detected);
                }
            }
        }

        Ok(repeat_detected)
    }

    async fn build_tool_arguments(
        &self,
        original_query: &str,
        server_id: &str,
        tool_name: &str,
        args: &serde_json::Value,
        args_valid: bool,
    ) -> serde_json::Value {
        let is_usable =
            args_valid && matches!(args, serde_json::Value::Object(map) if !map.is_empty());
        if is_usable {
            return args.clone();
        }

        if let Some(tool_def) = self.registry.get_tool_def(server_id, tool_name).await {
            return CapabilityRegistry::build_arguments_from_query(&tool_def, original_query);
        }

        serde_json::json!({ "query": original_query })
    }

    async fn append_tool_error(&self, session_id: &str, call_id: &str, error: String) {
        self.append_message(
            session_id,
            ChatMessage {
                role: "tool".to_string(),
                content: serde_json::json!({ "error": error }).to_string(),
                name: None,
                tool_call_id: Some(call_id.to_string()),
                tool_calls: None,
            },
        )
        .await;
    }

    // ══════════════════════════════════════════════════════════════
    //  PLAIN STREAMING
    // ══════════════════════════════════════════════════════════════

    async fn run_streaming(
        &self,
        session_id: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
        let effective_max_tokens = clamp_max_tokens(ctx_size, max_tokens);
        let prompt_budget = compute_prompt_budget(ctx_size, effective_max_tokens);
        let request_messages =
            sanitize_messages_for_request(trim_messages_to_budget(&messages, prompt_budget));

        let mut rx = self
            .service
            .send_chat_message(
                Some(session_id.to_string()),
                request_messages,
                temperature,
                0.95,
                40,
                effective_max_tokens,
            )
            .await?;

        let mut full_response = String::new();
        let mut parser = ThinkingStreamParser::new();

        while let Some(chunk) = rx.recv().await {
            for parsed in parser.push(&chunk) {
                match parsed {
                    ParsedChunk::Content(text) => {
                        full_response.push_str(&text);
                        if !Self::try_send(&on_event, serde_json::json!({ "chunk": text })) {
                            return Ok(());
                        }
                    }
                    ParsedChunk::Thinking(text) => {
                        if !Self::try_send(&on_event, serde_json::json!({ "thinking_chunk": text }))
                        {
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Flush any buffered content at end-of-stream
        for parsed in parser.flush() {
            match parsed {
                ParsedChunk::Content(text) => {
                    full_response.push_str(&text);
                    if !Self::try_send(&on_event, serde_json::json!({ "chunk": text })) {
                        return Ok(());
                    }
                }
                ParsedChunk::Thinking(text) => {
                    if !Self::try_send(&on_event, serde_json::json!({ "thinking_chunk": text })) {
                        return Ok(());
                    }
                }
            }
        }

        if !Self::try_send(&on_event, serde_json::json!({ "status": "done" })) {
            return Ok(());
        }

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

    // ══════════════════════════════════════════════════════════════
    //  SESSION MANAGEMENT
    // ══════════════════════════════════════════════════════════════

    async fn get_history(&self, session_id: &str) -> Vec<ChatMessage> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned().unwrap_or_default()
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

    pub async fn remove_message(
        &self,
        session_id: &str,
        message_index: usize,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let history = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if message_index >= history.len() {
            return Err("Message not found".to_string());
        }

        history.remove(message_index);
        Ok(())
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

        let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
        let effective_max_tokens = clamp_max_tokens(ctx_size, max_tokens);
        let prompt_budget = compute_prompt_budget(ctx_size, effective_max_tokens);
        let request_messages =
            sanitize_messages_for_request(trim_messages_to_budget(&history_before, prompt_budget));

        let mut rx = self
            .service
            .send_chat_message(
                Some(session_id.to_string()),
                request_messages,
                temperature,
                0.95,
                40,
                effective_max_tokens,
            )
            .await?;

        let mut full_response = String::new();
        let mut parser = ThinkingStreamParser::new();

        while let Some(chunk) = rx.recv().await {
            for parsed in parser.push(&chunk) {
                match parsed {
                    ParsedChunk::Content(text) => {
                        full_response.push_str(&text);
                        if !Self::try_send(&on_event, serde_json::json!({ "chunk": text })) {
                            return Ok(());
                        }
                    }
                    ParsedChunk::Thinking(text) => {
                        if !Self::try_send(&on_event, serde_json::json!({ "thinking_chunk": text }))
                        {
                            return Ok(());
                        }
                    }
                }
            }
        }

        for parsed in parser.flush() {
            match parsed {
                ParsedChunk::Content(text) => {
                    full_response.push_str(&text);
                    if !Self::try_send(&on_event, serde_json::json!({ "chunk": text })) {
                        return Ok(());
                    }
                }
                ParsedChunk::Thinking(text) => {
                    if !Self::try_send(&on_event, serde_json::json!({ "thinking_chunk": text })) {
                        return Ok(());
                    }
                }
            }
        }

        if !Self::try_send(&on_event, serde_json::json!({ "status": "done" })) {
            return Ok(());
        }

        let mut sessions = self.sessions.lock().await;
        let history = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if message_index >= history.len() {
            return Err("Message removed".to_string());
        }

        history[message_index].content = full_response;

        Ok(())
    }

    async fn current_ctx_size(&self) -> Option<u32> {
        self.service.get_config().await.map(|cfg| cfg.ctx_size)
    }
}

// ══════════════════════════════════════════════════════════════════
//  PURE FUNCTIONS
// ══════════════════════════════════════════════════════════════════

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

#[derive(Debug, Clone)]
struct LlmToolCall {
    id: String,
    tool_id: String,
    arguments: serde_json::Value,
    arguments_valid: bool,
}

#[derive(Debug, Clone)]
struct ParsedToolCalls {
    content: String,
    raw_tool_calls: Vec<serde_json::Value>,
    tool_calls: Vec<LlmToolCall>,
}

fn parse_tool_calls_from_response(response: &serde_json::Value) -> Result<ParsedToolCalls, String> {
    let message = response
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("message"))
        .ok_or_else(|| "Missing response message".to_string())?;

    let content = message
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
        let raw_tool_calls = tool_calls.to_vec();
        let mut parsed = Vec::new();
        for (idx, call) in tool_calls.iter().enumerate() {
            if let Some(parsed_call) = parse_tool_call_entry(call, idx) {
                parsed.push(parsed_call);
            }
        }
        return Ok(ParsedToolCalls {
            content,
            raw_tool_calls,
            tool_calls: parsed,
        });
    }

    if let Some(function_call) = message.get("function_call") {
        if let Some(parsed_call) = parse_function_call(function_call, 0) {
            return Ok(ParsedToolCalls {
                content,
                raw_tool_calls: vec![serde_json::json!({
                    "id": parsed_call.id,
                    "type": "function",
                    "function": {
                        "name": parsed_call.tool_id,
                        "arguments": parsed_call.arguments
                    }
                })],
                tool_calls: vec![parsed_call],
            });
        }
    }

    Ok(ParsedToolCalls {
        content,
        raw_tool_calls: Vec::new(),
        tool_calls: Vec::new(),
    })
}

fn parse_tool_call_entry(call: &serde_json::Value, idx: usize) -> Option<LlmToolCall> {
    let call_id = call
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default_tool_call_id(idx));
    let function = call.get("function")?;
    parse_function_call(function, idx).map(|mut parsed| {
        parsed.id = call_id;
        parsed
    })
}

fn parse_function_call(function: &serde_json::Value, idx: usize) -> Option<LlmToolCall> {
    let tool_id = function.get("name")?.as_str()?.to_string();
    let args_val = function
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let (arguments, arguments_valid) = parse_tool_arguments(&args_val);
    Some(LlmToolCall {
        id: default_tool_call_id(idx),
        tool_id,
        arguments,
        arguments_valid,
    })
}

fn default_tool_call_id(idx: usize) -> String {
    format!("call-{}", idx)
}

fn parse_tool_arguments(value: &serde_json::Value) -> (serde_json::Value, bool) {
    match value {
        serde_json::Value::Null => (serde_json::json!({}), false),
        serde_json::Value::Object(_) => (value.clone(), true),
        serde_json::Value::String(s) => match serde_json::from_str::<serde_json::Value>(s) {
            Ok(serde_json::Value::Object(obj)) => (serde_json::Value::Object(obj), true),
            Ok(other) => (other, false),
            Err(_) => (serde_json::json!({}), false),
        },
        _ => (serde_json::json!({}), false),
    }
}

fn resolve_tool_id(tool_id: &str, tool_bundle: &LlmToolSpecBundle) -> Option<(String, String)> {
    if let Some(pair) = tool_bundle.tool_map.get(tool_id) {
        return Some(pair.clone());
    }
    None
}

fn compute_prompt_budget(ctx_size: usize, max_tokens: i32) -> usize {
    let completion_budget = max_tokens.max(256) as usize;
    ctx_size.saturating_sub(completion_budget + 128).max(512)
}

fn clamp_max_tokens(ctx_size: usize, requested: i32) -> i32 {
    let upper = ctx_size.saturating_sub(256).max(256) as i32;
    requested.clamp(32, upper)
}

fn estimate_message_tokens(message: &ChatMessage) -> usize {
    8 + (message.content.chars().count() / 4).max(1)
}

fn trim_messages_to_budget(messages: &[ChatMessage], budget: usize) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }

    let mut selected_rev: Vec<ChatMessage> = Vec::new();
    let mut used = 0usize;

    for message in messages.iter().rev() {
        let tokens = estimate_message_tokens(message);
        if !selected_rev.is_empty() && used + tokens > budget {
            break;
        }
        used += tokens;
        selected_rev.push(message.clone());
    }

    selected_rev.reverse();
    selected_rev
}

/// Truncate a string to fit approximately within a token budget.
/// Uses the ~4 chars per token heuristic. Appends a truncation notice
/// so the LLM knows the data was cut.
fn truncate_to_token_budget(text: &str, token_budget: usize) -> String {
    let max_chars = token_budget.max(64) * 4;
    if text.len() <= max_chars {
        return text.to_string();
    }

    // Find a safe cut point (don't split mid-line if possible)
    let cut = text[..max_chars].rfind('\n').unwrap_or(max_chars);

    let mut truncated = text[..cut].to_string();
    truncated.push_str("\n\n[... data truncated to fit context window ...]");
    truncated
}

fn sanitize_messages_for_request(mut messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    // llama.cpp rejects payloads that end with multiple assistant messages.
    while messages.len() >= 2 {
        let len = messages.len();
        if messages[len - 1].role == "assistant" && messages[len - 2].role == "assistant" {
            messages.remove(len - 2);
            continue;
        }
        break;
    }
    messages
}

fn format_tool_result(result: &serde_json::Value) -> String {
    let json_str = serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string());
    truncate_to_token_budget(&json_str, TOOL_RESULT_TOKEN_BUDGET)
}

#[cfg(test)]
mod tests {
    use super::{parse_tool_arguments, parse_tool_calls_from_response};

    #[test]
    fn parse_tool_arguments_accepts_json_string() {
        let (args, ok) = parse_tool_arguments(&serde_json::json!("{\"x\":1}"));
        assert!(ok);
        assert_eq!(args["x"], 1);
    }

    #[test]
    fn parse_tool_calls_extracts_function_call() {
        let response = serde_json::json!({
            "choices": [
                { "message": { "content": "", "tool_calls": [
                    { "id": "call-1", "type": "function", "function": { "name": "mcp__s1__t1", "arguments": "{\"q\":\"hi\"}" } }
                ] } }
            ]
        });

        let parsed = parse_tool_calls_from_response(&response).expect("parsed");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].tool_id, "mcp__s1__t1");
        assert_eq!(parsed.tool_calls[0].arguments["q"], "hi");
    }
}
