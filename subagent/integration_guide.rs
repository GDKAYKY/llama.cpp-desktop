// ══════════════════════════════════════════════════════════════════
// INTEGRATION GUIDE: Adding Subagent to ChatOrchestrator
// ══════════════════════════════════════════════════════════════════

// 1. ADD TO IMPORTS AT TOP OF orchestrator.rs:
use crate::services::subagent::{Subagent, SubagentResult, format_subagent_data_for_prompt};

// 2. ADD FIELD TO ChatOrchestrator struct:
pub struct ChatOrchestrator {
    sessions: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    service: LlamaCppService,
    mcp_service: McpService,
    registry: CapabilityRegistry,
    subagent: Subagent,  // <-- ADD THIS
}

// 3. UPDATE THE CONSTRUCTOR:
impl ChatOrchestrator {
    pub fn new(service: LlamaCppService, mcp_service: McpService) -> Self {
        let registry = CapabilityRegistry::new();
        let subagent = Subagent::new(
            service.clone(),
            mcp_service.clone(),
            Arc::new(registry.clone()), // Share registry with subagent
        );
        
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            service,
            mcp_service,
            registry,
            subagent,
        }
    }
    
    // ... rest of implementation
}

// 4. MODIFY classify_intent TO DETECT MULTI-STEP NEEDS:
// Add a new field to IntentClassification:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub needs_external: bool,
    pub query: String,
    pub suggested_tool: Option<String>,
    pub suggested_server: Option<String>,
    pub arguments: Option<serde_json::Value>,
    pub needs_multi_step: bool,  // <-- ADD THIS
    pub multi_step_reasoning: Option<String>,  // <-- ADD THIS
}

// Update the system prompt in classify_intent:
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
  "needs_multi_step": boolean,  // NEW: true if query needs multiple tool calls
  "multi_step_reasoning": "why multiple steps are needed or null"  // NEW
}}

Available capabilities:
{}

Rules:
- If the user's request can be answered from general knowledge, set needs_external=false.
- If external data/action is needed, set needs_external=true and fill query + suggested_tool.
- Set needs_multi_step=true if:
  * Query needs data from multiple sources
  * Results from one tool will inform what to call next
  * Task requires iteration or multiple lookups
- For multi-step tasks, provide multi_step_reasoning explaining why
- arguments should match the tool's expected input schema.
- Do NOT invent tool names. Only use names from the list above.
- Return ONLY the JSON object, nothing else."#,
        capabilities_summary
    );

    // ... rest of classify_intent implementation
}

// 5. MODIFY process() METHOD TO USE SUBAGENT:
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
    if !allowed_servers.is_empty() {
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
            // ★ NEW: Check if multi-step is needed
            if intent.needs_multi_step {
                let _ = on_event.send(serde_json::json!({
                    "thinking": format!(
                        "Multi-step task detected: {}. Activating subagent...",
                        intent.multi_step_reasoning.as_deref().unwrap_or("complex query")
                    )
                }));

                // Run the subagent loop
                match self.subagent.execute(&intent.query, &allowed_servers, temperature).await {
                    Ok(subagent_result) => {
                        let _ = on_event.send(serde_json::json!({
                            "thinking": format!(
                                "Subagent completed: {} tool calls in {} iterations",
                                subagent_result.data.len(),
                                subagent_result.iterations_used
                            )
                        }));

                        // Format the gathered data
                        let formatted_data = format_subagent_data_for_prompt(&subagent_result);

                        // Answer using all gathered data
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
                            "thinking": format!("Subagent failed: {}. Falling back to single tool.", e)
                        }));
                        // Fall through to single-tool logic below
                    }
                }
            }

            // ── Single-tool path (existing logic) ──
            let resolved = self.resolve_tool(&intent, &allowed_servers).await;

            if let Some(call) = resolved {
                self.registry.validate_call(&call).await?;

                let _ = on_event.send(serde_json::json!({
                    "thinking": format!(
                        "Single tool dispatch: server='{}', tool='{}'",
                        call.server_id, call.tool_name
                    )
                }));

                self.mcp_service.connect(&call.server_id).await?;
                let mcp_result = self
                    .mcp_service
                    .tools_call(&call.server_id, &call.tool_name, call.arguments.clone())
                    .await?;

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

// 6. ADD NEW METHOD FOR SUBAGENT DATA:
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
    let history_tokens: usize = history.iter().map(|m| estimate_message_tokens(m)).sum();
    let system_overhead = 100;
    let safety_margin = 128;

    let available_for_data = ctx_size
        .saturating_sub(history_tokens)
        .saturating_sub(effective_max_tokens)
        .saturating_sub(system_overhead)
        .saturating_sub(safety_margin);

    // Truncate if needed
    let truncated_data = truncate_to_token_budget(formatted_data, available_for_data);

    // Build prompt with subagent data
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: format!(
            "A specialized subagent has gathered the following external data for you.\n\
             Use this data to provide a comprehensive answer to the user's question.\n\
             Synthesize information from multiple sources if available.\n\n\
             {}\n\n\
             Answer naturally in the user's language. Be thorough but concise.",
            truncated_data
        ),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    messages.extend(history);

    self.run_streaming(session_id, messages, temperature, max_tokens, on_event)
        .await
}

// ══════════════════════════════════════════════════════════════════
// USAGE EXAMPLES
// ══════════════════════════════════════════════════════════════════

// Example 1: Simple query (single tool)
// User: "What's the weather in Tokyo?"
// Flow: classify → single tool call → answer

// Example 2: Multi-step query (subagent activates)
// User: "Compare the weather in Tokyo, New York, and London"
// Flow: 
//   1. classify → needs_multi_step=true
//   2. subagent iteration 1: call weather(Tokyo)
//   3. subagent iteration 2: call weather(New York)
//   4. subagent iteration 3: call weather(London)
//   5. subagent finishes with summary
//   6. main orchestrator uses all data to answer

// Example 3: Complex research query
// User: "Find recent news about AI, then search for papers on those topics"
// Flow:
//   1. classify → needs_multi_step=true
//   2. subagent iteration 1: search_news("AI recent")
//   3. subagent iteration 2: extract_topics(news_result)
//   4. subagent iteration 3: search_papers(topic1)
//   5. subagent iteration 4: search_papers(topic2)
//   6. subagent finishes
//   7. main orchestrator synthesizes everything
