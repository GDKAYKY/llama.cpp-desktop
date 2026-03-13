use crate::models::{ChatMessage, IntentClassification};
use crate::services::capability_registry::CapabilityRegistry;
use crate::services::capability_registry::ResolvedCall;
use crate::services::llama::service::LlamaCppService;
use crate::services::mcp::McpService;
use crate::services::subagent::{format_subagent_data_for_prompt, Subagent};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ChatOrchestrator {
    sessions: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    service: LlamaCppService,
    mcp_service: McpService,
    registry: CapabilityRegistry,
    subagent: Subagent,
}

impl ChatOrchestrator {
    pub fn new(service: LlamaCppService, mcp_service: McpService) -> Self {
        let registry = CapabilityRegistry::new();
        let subagent = Subagent::new(service.clone(), mcp_service.clone(), registry.clone());

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            service,
            mcp_service,
            registry,
            subagent,
        }
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
        let (mentioned_mcp_ids, cleaned_input) = extract_mcp_ids(&user_input);
        let allowed_servers = if !mentioned_mcp_ids.is_empty() {
            mentioned_mcp_ids
        } else {
            self.registry.available_server_ids().await
        };

        // Push user message to history
        {
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
        }

        // ── Step 1: Intent classification (cheap, fast) ──
        // Skip MCP entirely for internal sessions (title generation, etc.)
        let is_internal_session = session_id.starts_with("summary-");
        if !is_internal_session && !allowed_servers.is_empty() {
            let intent = self
                .classify_intent(
                    session_id,
                    &cleaned_input,
                    &allowed_servers,
                    temperature,
                    max_tokens,
                )
                .await?;

            let _ = on_event.send(serde_json::json!({
                "thinking": format!(
                    "Intent: needs_external={}, needs_multi_step={}, query='{}'",
                    intent.needs_external, intent.needs_multi_step, intent.query
                )
            }));

            if intent.needs_external {
                if intent.needs_multi_step {
                    let _ = on_event.send(serde_json::json!({
                        "thinking": format!(
                            "Multi-step task detected: {}. Activating subagent...",
                            intent.multi_step_reasoning.as_deref().unwrap_or("complex query")
                        )
                    }));

                    match self
                        .subagent
                        .execute(&intent.query, &allowed_servers, temperature)
                        .await
                    {
                        Ok(subagent_result) => {
                            let _ = on_event.send(serde_json::json!({
                                "thinking": format!(
                                    "Subagent completed: {} tool calls in {} iterations",
                                    subagent_result.data.len(),
                                    subagent_result.iterations_used
                                )
                            }));

                            let formatted_data =
                                format_subagent_data_for_prompt(&subagent_result);

                            return self
                                .answer_with_formatted_data(
                                    session_id,
                                    &formatted_data,
                                    temperature,
                                    max_tokens,
                                    on_event,
                                )
                                .await;
                        }
                        Err(e) => {
                            let _ = on_event.send(serde_json::json!({
                                "thinking": format!(
                                    "Subagent failed: {}. Falling back to single-tool path.",
                                    e
                                )
                            }));
                        }
                    }
                }

                // ── Step 2: Host resolves tool (deterministic) ──
                let resolved = self.resolve_tool(&intent, &allowed_servers).await;

                if let Some(call) = resolved {
                    // ── Step 3: Hard validation ──
                    self.registry.validate_call(&call).await?;

                    let _ = on_event.send(serde_json::json!({
                        "thinking": format!(
                            "Dispatching: server='{}', tool='{}' (host-validated)",
                            call.server_id, call.tool_name
                        )
                    }));

                    // ── Step 4: Execute MCP (deterministic) ──
                    self.mcp_service.connect(&call.server_id).await?;
                    let mcp_result = self
                        .mcp_service
                        .tools_call(&call.server_id, &call.tool_name, call.arguments.clone())
                        .await?;
                    let extracted_text = extract_mcp_result_text(&mcp_result);
                    let data_preview_len = extracted_text.chars().count();
                    let data_preview: String = extracted_text.chars().take(220).collect();
                    let top_level_keys = mcp_result
                        .as_object()
                        .map(|obj| {
                            obj.keys()
                                .take(4)
                                .cloned()
                                .collect::<Vec<String>>()
                                .join(", ")
                        })
                        .unwrap_or_else(|| "non-object".to_string());
                    let _ = on_event.send(serde_json::json!({
                        "thinking": format!(
                            "MCP result received: {} chars, keys: {}",
                            data_preview_len, top_level_keys
                        )
                    }));
                    let _ = on_event.send(serde_json::json!({
                        "thinking": format!("MCP preview: {}", data_preview.replace('\n', " "))
                    }));

                    // ── Step 5: LLM answers with data ──
                    return self
                        .answer_with_data(
                            session_id,
                            &mcp_result,
                            temperature,
                            max_tokens,
                            on_event,
                        )
                        .await;
                } else {
                    let _ = on_event.send(serde_json::json!({
                        "thinking": "No matching tool found in registry; falling back to plain LLM."
                    }));
                }
            }
        }

        // ── Fallback: plain streaming (no MCP) ──
        let messages = self.get_history(session_id).await;
        self.run_streaming(session_id, messages, temperature, max_tokens, on_event)
            .await
    }

    // ══════════════════════════════════════════════════════════════
    //  INTENT CLASSIFICATION — the ONLY thing the LLM decides
    // ══════════════════════════════════════════════════════════════

    async fn classify_intent(
        &self,
        session_id: &str,
        user_input: &str,
        allowed_servers: &[String],
        temperature: f32,
        _max_tokens: i32,
    ) -> Result<IntentClassification, String> {
        let capabilities_summary = self.registry.summary_for_prompt(allowed_servers).await;

        let system_prompt = format!(
            r#"You are an intent classifier. Given the user message and available tools, return ONLY valid JSON:
{{
  "needs_external": boolean,
  "query": "the search/action query extracted from user message",
  "suggested_tool": "tool_name or null",
  "suggested_server": "server_id or null",
  "arguments": {{}} or null,
  "needs_multi_step": boolean,
  "multi_step_reasoning": "why multiple steps are needed or null"
}}

Available capabilities:
{}

Rules:
- If the user's request can be answered from general knowledge, set needs_external=false.
- If external data/action is needed, set needs_external=true and fill query + suggested_tool.
- Set needs_multi_step=true when:
  * multiple entities/sources must be queried
  * one tool result determines a later call
  * iterative lookups are required
- If needs_multi_step=true, provide concise multi_step_reasoning.
- arguments should match the tool's expected input schema.
- Do NOT invent tool names. Only use names from the list above.
- Return ONLY the JSON object, nothing else."#,
            capabilities_summary
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_input.to_string(),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
        let effective_max_tokens = clamp_max_tokens(ctx_size, 256); // intent is tiny

        // Use None session to avoid polluting the main conversation's KV-cache slot.
        // Intent classification is a one-shot ephemeral call.
        let response = self
            .service
            .complete_chat(
                None,
                messages,
                temperature.min(0.3), // low temp for classification
                0.95,
                40,
                effective_max_tokens,
                None,
                None,
            )
            .await?;

        let content = response
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Try to parse; fall back to needs_external=false
        Ok(parse_intent(content, user_input))
    }

    // ══════════════════════════════════════════════════════════════
    //  DETERMINISTIC TOOL RESOLUTION (host logic, not LLM)
    // ══════════════════════════════════════════════════════════════

    async fn resolve_tool(
        &self,
        intent: &IntentClassification,
        allowed_servers: &[String],
    ) -> Option<ResolvedCall> {
        // Priority 1: LLM suggested a tool + server — validate both exist
        if let (Some(tool), Some(server)) = (&intent.suggested_tool, &intent.suggested_server) {
            if self.registry.has_tool(server, tool).await {
                let arguments = self.build_call_arguments(server, tool, intent).await;
                return Some(ResolvedCall {
                    server_id: server.clone(),
                    tool_name: tool.clone(),
                    arguments,
                });
            }
        }

        // Priority 2: LLM suggested tool name but wrong/missing server
        if let Some(tool) = &intent.suggested_tool {
            for server_id in allowed_servers {
                if self.registry.has_tool(server_id, tool).await {
                    let arguments = self.build_call_arguments(server_id, tool, intent).await;
                    return Some(ResolvedCall {
                        server_id: server_id.clone(),
                        tool_name: tool.clone(),
                        arguments,
                    });
                }
            }
        }

        // Priority 3: keyword matching against registry
        if let Some(mut call) = self
            .registry
            .match_tool(&intent.query, allowed_servers)
            .await
        {
            let arguments = self
                .build_call_arguments(&call.server_id, &call.tool_name, intent)
                .await;
            call.arguments = arguments;
            return Some(call);
        }

        None
    }

    /// Build arguments for a tool call. Uses LLM-provided arguments if they
    /// contain non-empty fields, otherwise derives arguments from the tool's
    /// input schema + the extracted query string.
    async fn build_call_arguments(
        &self,
        server_id: &str,
        tool_name: &str,
        intent: &IntentClassification,
    ) -> serde_json::Value {
        // If the LLM provided non-empty arguments, use them
        if let Some(args) = &intent.arguments {
            let is_usable = match args {
                serde_json::Value::Object(map) => !map.is_empty(),
                _ => false,
            };
            if is_usable {
                return args.clone();
            }
        }

        // Otherwise, build arguments from the tool's schema + query
        if let Some(tool_def) = self.registry.get_tool_def(server_id, tool_name).await {
            return CapabilityRegistry::build_arguments_from_query(&tool_def, &intent.query);
        }

        // Ultimate fallback
        serde_json::json!({ "query": intent.query })
    }

    // ══════════════════════════════════════════════════════════════
    //  ANSWER WITH DATA — LLM does what it's good at: language
    // ══════════════════════════════════════════════════════════════

    async fn answer_with_data(
        &self,
        session_id: &str,
        mcp_result: &serde_json::Value,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
        let effective_max_tokens = clamp_max_tokens(ctx_size, max_tokens) as usize;

        // Calculate how much room we have for the MCP data
        let history = self.get_history(session_id).await;
        let history_tokens: usize = history.iter().map(|m| estimate_message_tokens(m)).sum();
        let system_overhead = 80; // tokens for the surrounding system prompt text
        let safety_margin = 128; // buffer for templates/metadata

        let available_for_data = ctx_size
            .saturating_sub(history_tokens)
            .saturating_sub(effective_max_tokens)
            .saturating_sub(system_overhead)
            .saturating_sub(safety_margin);

        let data_str = extract_mcp_result_text(mcp_result);

        // Truncate the data to fit within the available budget
        let truncated_data = truncate_to_token_budget(&data_str, available_for_data);

        // Build a focused prompt for MCP answers.
        // Use only the latest user request + external data to avoid stale assistant
        // replies ("I can't access tools") contaminating the response.
        let mut messages = vec![ChatMessage {
            role: "system".to_string(),
            content: format!(
                "You already have external data retrieved by the host.\n\
                 Never say you cannot access external tools, services, or live data.\n\
                 Use the external data below as the primary source for your answer.\n\
                 If data is incomplete, say what is missing briefly.\n\
                 External data:\n{}\n\n\
                 Answer naturally in the user's language. Do not mention JSON, tools, or MCP.",
                truncated_data
            ),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }];
        let last_user_request = history
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "Answer the user's latest request.".to_string());
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!(
                "User request:\n{}\n\nProvide the best possible answer using the external data.",
                last_user_request
            ),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });

        self.run_streaming(session_id, messages, temperature, max_tokens, on_event)
            .await
    }

    async fn answer_with_formatted_data(
        &self,
        session_id: &str,
        formatted_data: &str,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let ctx_size = self.current_ctx_size().await.unwrap_or(4096) as usize;
        let effective_max_tokens = clamp_max_tokens(ctx_size, max_tokens) as usize;

        let history = self.get_history(session_id).await;
        let history_tokens: usize = history.iter().map(estimate_message_tokens).sum();
        let system_overhead = 100;
        let safety_margin = 128;

        let available_for_data = ctx_size
            .saturating_sub(history_tokens)
            .saturating_sub(effective_max_tokens)
            .saturating_sub(system_overhead)
            .saturating_sub(safety_margin);

        let truncated_data = truncate_to_token_budget(formatted_data, available_for_data);
        let last_user_request = history
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "Answer the user's latest request.".to_string());

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "You already have external data retrieved by a specialized subagent.\n\
                     Never say you cannot access external tools, services, or live data.\n\
                     Use the external data below as the primary source for your answer.\n\
                     If data is incomplete, say what is missing briefly.\n\
                     External data:\n{}\n\n\
                     Answer naturally in the user's language. Do not mention JSON, tools, or MCP.",
                    truncated_data
                ),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "User request:\n{}\n\nProvide the best possible answer using the external data.",
                    last_user_request
                ),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        self.run_streaming(session_id, messages, temperature, max_tokens, on_event)
            .await
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

        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
            let _ = on_event.send(serde_json::json!({ "chunk": chunk }));
        }

        let _ = on_event.send(serde_json::json!({ "status": "done" }));

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

        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
            let _ = on_event.send(serde_json::json!({ "chunk": chunk }));
        }

        let _ = on_event.send(serde_json::json!({ "status": "done" }));

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

fn parse_intent(content: &str, original_query: &str) -> IntentClassification {
    let trimmed = content.trim();

    // Strip Qwen3 / thinking-mode <think>...</think> wrapper before parsing.
    // The model emits <think>...reasoning...</think>\n{...json...}
    let stripped = if let (Some(open), Some(close)) = (trimmed.find("<think>"), trimmed.find("</think>")) {
        if close > open {
            trimmed[close + "</think>".len()..].trim()
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    // Try direct parse
    if let Ok(intent) = serde_json::from_str::<IntentClassification>(stripped) {
        return intent;
    }

    // Try extracting fenced JSON block (```json ... ```)
    if let Some(fence_start) = stripped.find("```") {
        let after_fence = &stripped[fence_start + 3..];
        let after_lang = after_fence
            .trim_start_matches(|c: char| c.is_alphabetic())
            .trim_start_matches('\n');
        if let Some(fence_end) = after_lang.find("```") {
            let candidate = after_lang[..fence_end].trim();
            if let Ok(intent) = serde_json::from_str::<IntentClassification>(candidate) {
                return intent;
            }
        }
    }

    // Try extracting bare JSON object
    if let Some(start) = stripped.find('{') {
        if let Some(end) = stripped.rfind('}') {
            if end > start {
                if let Ok(intent) =
                    serde_json::from_str::<IntentClassification>(&stripped[start..=end])
                {
                    return intent;
                }
            }
        }
    }

    // Safe fallback: no external data needed
    IntentClassification {
        needs_external: false,
        query: original_query.to_string(),
        suggested_tool: None,
        suggested_server: None,
        arguments: None,
        needs_multi_step: false,
        multi_step_reasoning: None,
    }
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

fn extract_mcp_result_text(mcp_result: &serde_json::Value) -> String {
    // Prefer structured payloads first (e.g., Tavily `structuredContent`),
    // because `content.text` is often a JSON blob encoded as string.
    if let Some(structured) = mcp_result.get("structuredContent") {
        // Special formatting for search-style results:
        // { query, answer?, results: [{ title, url, content, score? }, ...] }
        if let Some(obj) = structured.as_object() {
            let mut out = String::new();

            if let Some(query) = obj.get("query").and_then(|v| v.as_str()) {
                out.push_str("Query: ");
                out.push_str(query);
                out.push_str("\n\n");
            }

            if let Some(answer) = obj.get("answer").and_then(|v| v.as_str()) {
                if !answer.trim().is_empty() {
                    out.push_str("Search answer:\n");
                    out.push_str(answer.trim());
                    out.push_str("\n\n");
                }
            }

            if let Some(results) = obj.get("results").and_then(|v| v.as_array()) {
                out.push_str("Top results:\n");
                for (idx, item) in results.iter().take(6).enumerate() {
                    let title = item
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(no title)");
                    let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    let content = item
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let content = if content.chars().count() > 420 {
                        let mut t: String = content.chars().take(420).collect();
                        t.push_str("...");
                        t
                    } else {
                        content.to_string()
                    };

                    out.push_str(&format!("{}. {}\n", idx + 1, title));
                    if !url.is_empty() {
                        out.push_str(&format!("   URL: {}\n", url));
                    }
                    if !content.is_empty() {
                        out.push_str(&format!("   Snippet: {}\n", content));
                    }
                }
                out.push('\n');
            }

            if !out.trim().is_empty() {
                return out;
            }
        }

        if let Ok(s) = serde_json::to_string_pretty(structured) {
            if !s.is_empty() {
                return s;
            }
        }
    }

    // Fallback for MCP servers that only return `content` blocks.
    if let Some(content_items) = mcp_result.get("content").and_then(|v| v.as_array()) {
        let texts: Vec<String> = content_items
            .iter()
            .filter_map(|item| {
                if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                    return item
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                None
            })
            .collect();
        if !texts.is_empty() {
            return texts.join("\n\n");
        }
    }

    serde_json::to_string_pretty(mcp_result).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_intent;

    #[test]
    fn parse_intent_defaults_when_multi_step_fields_missing() {
        let content = r#"{
            "needs_external": true,
            "query": "weather in tokyo",
            "suggested_tool": "weather",
            "suggested_server": "tavily",
            "arguments": {"location":"Tokyo"}
        }"#;

        let parsed = parse_intent(content, "fallback");
        assert!(parsed.needs_external);
        assert_eq!(parsed.query, "weather in tokyo");
        assert!(!parsed.needs_multi_step);
        assert_eq!(parsed.multi_step_reasoning, None);
    }

    #[test]
    fn parse_intent_reads_full_multi_step_fields() {
        let content = r#"{
            "needs_external": true,
            "query": "compare weather",
            "suggested_tool": "weather",
            "suggested_server": "tavily",
            "arguments": {"locations":["Tokyo","NYC"]},
            "needs_multi_step": true,
            "multi_step_reasoning": "must query multiple locations"
        }"#;

        let parsed = parse_intent(content, "fallback");
        assert!(parsed.needs_multi_step);
        assert_eq!(
            parsed.multi_step_reasoning.as_deref(),
            Some("must query multiple locations")
        );
    }

    #[test]
    fn parse_intent_fallback_sets_multi_step_defaults() {
        let parsed = parse_intent("not-json", "original");
        assert!(!parsed.needs_external);
        assert_eq!(parsed.query, "original");
        assert!(!parsed.needs_multi_step);
        assert_eq!(parsed.multi_step_reasoning, None);
    }
}
