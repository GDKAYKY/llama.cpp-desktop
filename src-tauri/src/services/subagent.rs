// Legacy subagent implementation (kept for reference).
use crate::models::ChatMessage;
use crate::services::capability_registry::{CapabilityRegistry, ResolvedCall};
use crate::services::llama::service::LlamaCppService;
use crate::services::mcp::McpService;
use serde::{Deserialize, Serialize};

const MAX_SUBAGENT_ITERATIONS: usize = 5;
const SUBAGENT_MAX_TOKENS: i32 = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentAction {
    pub action_type: String, // "call_tool" or "finish"
    pub reasoning: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_summary: Option<String>,
}

pub struct SubagentResult {
    pub data: Vec<ToolCallResult>,
    pub summary: String,
    pub iterations_used: usize,
}

#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub server_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: serde_json::Value,
}

#[derive(Clone)]
pub struct Subagent {
    service: LlamaCppService,
    mcp_service: McpService,
    registry: CapabilityRegistry,
}

impl Subagent {
    pub fn new(
        service: LlamaCppService,
        mcp_service: McpService,
        registry: CapabilityRegistry,
    ) -> Self {
        Self {
            service,
            mcp_service,
            registry,
        }
    }

    /// Main entry point: runs the subagent loop to gather all needed data
    pub async fn execute(
        &self,
        original_query: &str,
        allowed_servers: &[String],
        temperature: f32,
    ) -> Result<SubagentResult, String> {
        let mut iteration = 0;
        let mut tool_results: Vec<ToolCallResult> = Vec::new();
        let mut conversation_history: Vec<ChatMessage> = Vec::new();

        // Initial system prompt for the subagent
        let system_prompt = self
            .build_system_prompt(original_query, allowed_servers)
            .await;
        conversation_history.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });

        // Initial user message
        conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: format!("Task: {}", original_query),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });

        // Agentic loop
        loop {
            iteration += 1;

            if iteration > MAX_SUBAGENT_ITERATIONS {
                return Err(format!(
                    "Subagent exceeded max iterations ({})",
                    MAX_SUBAGENT_ITERATIONS
                ));
            }

            // Ask the subagent what to do next
            let action = self
                .plan_next_action(&conversation_history, temperature)
                .await?;

            match action.action_type.as_str() {
                "finish" => {
                    let summary = action
                        .final_summary
                        .unwrap_or_else(|| self.generate_summary(&tool_results));

                    return Ok(SubagentResult {
                        data: tool_results,
                        summary,
                        iterations_used: iteration,
                    });
                }
                "call_tool" => {
                    // Execute the tool call
                    let tool_result = self.execute_tool_call(&action, allowed_servers).await?;

                    // Add to history for context
                    let result_summary = format!(
                        "Tool called: {} from server {}\nArguments: {}\nResult: {}",
                        tool_result.tool_name,
                        tool_result.server_id,
                        serde_json::to_string_pretty(&tool_result.arguments).unwrap_or_default(),
                        self.summarize_tool_result(&tool_result.result)
                    );

                    conversation_history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "Action: {}\nReasoning: {}",
                            action.action_type, action.reasoning
                        ),
                        name: None,
                        tool_call_id: None,
                        tool_calls: None,
                    });

                    conversation_history.push(ChatMessage {
                        role: "user".to_string(),
                        content: result_summary.clone(),
                        name: None,
                        tool_call_id: None,
                        tool_calls: None,
                    });

                    tool_results.push(tool_result);
                }
                _ => {
                    return Err(format!("Unknown action type: {}", action.action_type));
                }
            }
        }
    }

    /// Ask the LLM to plan the next action
    async fn plan_next_action(
        &self,
        conversation_history: &[ChatMessage],
        temperature: f32,
    ) -> Result<SubagentAction, String> {
        let response = self
            .service
            .complete_chat(
                None, // Ephemeral session
                conversation_history.to_vec(),
                temperature.min(0.5),
                0.95,
                40,
                SUBAGENT_MAX_TOKENS,
                None,
                None,
                None,
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

        self.parse_action(content)
    }

    /// Execute a single tool call
    async fn execute_tool_call(
        &self,
        action: &SubagentAction,
        allowed_servers: &[String],
    ) -> Result<ToolCallResult, String> {
        let tool_name = action
            .tool_name
            .as_ref()
            .ok_or("Missing tool_name in call_tool action")?;
        let server_id = action
            .server_id
            .as_ref()
            .ok_or("Missing server_id in call_tool action")?;

        // Validate the call
        if !allowed_servers.contains(server_id) {
            return Err(format!("Server '{}' not in allowed list", server_id));
        }

        let resolved_call = ResolvedCall {
            server_id: server_id.clone(),
            tool_name: tool_name.clone(),
            arguments: action.arguments.clone().unwrap_or(serde_json::json!({})),
        };

        self.registry.validate_call(&resolved_call).await?;

        // Connect and execute
        self.mcp_service.connect(server_id).await?;
        let result = self
            .mcp_service
            .tools_call(server_id, tool_name, resolved_call.arguments.clone())
            .await?;

        Ok(ToolCallResult {
            server_id: server_id.clone(),
            tool_name: tool_name.clone(),
            arguments: resolved_call.arguments,
            result,
        })
    }

    /// Build the system prompt for the subagent
    async fn build_system_prompt(
        &self,
        original_query: &str,
        allowed_servers: &[String],
    ) -> String {
        let capabilities = self
            .registry
            .summary_for_prompt_json(allowed_servers, None)
            .await;

        format!(
            r#"You are a data-gathering subagent. Your job is to collect all necessary information to answer the user's query by calling external tools.

Original query: {}

Available tools (JSON):
{}

RESPONSE FORMAT (JSON only):
{{
  "action_type": "call_tool" OR "finish",
  "reasoning": "brief explanation of your decision",
  "tool_name": "tool_name_here",      // required if action_type is "call_tool"
  "server_id": "server_id_here",      // required if action_type is "call_tool"
  "arguments": {{}},                   // required if action_type is "call_tool"
  "final_summary": "summary text"     // required if action_type is "finish"
}}

RULES:
1. Start by calling the most relevant tool to gather initial data
2. After each tool result, decide if you have enough information
3. If you need more data, call another tool with "call_tool"
4. When you have all needed information, use "finish" with a final_summary
5. The final_summary should be a clear, structured summary of all gathered data
6. You can call up to {} tools total
7. Only use tool names and server IDs from the list above

Return ONLY valid JSON, nothing else."#,
            original_query, capabilities, MAX_SUBAGENT_ITERATIONS
        )
    }

    /// Parse the LLM's action response
    fn parse_action(&self, content: &str) -> Result<SubagentAction, String> {
        parse_action_content(content)
    }

    /// Summarize a tool result for the conversation history
    fn summarize_tool_result(&self, result: &serde_json::Value) -> String {
        let json_str = serde_json::to_string_pretty(result).unwrap_or_default();

        // Truncate if too long
        if json_str.len() > 1000 {
            format!("{}... (truncated)", &json_str[..1000])
        } else {
            json_str
        }
    }

    /// Generate a default summary if the LLM doesn't provide one
    fn generate_summary(&self, tool_results: &[ToolCallResult]) -> String {
        if tool_results.is_empty() {
            return "No data was gathered.".to_string();
        }

        let mut summary = format!(
            "Gathered data from {} tool call(s):\n\n",
            tool_results.len()
        );

        for (idx, result) in tool_results.iter().enumerate() {
            summary.push_str(&format!(
                "{}. {} ({}): {}\n",
                idx + 1,
                result.tool_name,
                result.server_id,
                self.summarize_tool_result(&result.result)
            ));
        }

        summary
    }
}

fn parse_action_content(content: &str) -> Result<SubagentAction, String> {
    let trimmed = content.trim();

    // Try direct parse
    if let Ok(action) = serde_json::from_str::<SubagentAction>(trimmed) {
        return Ok(action);
    }

    // Try extracting JSON from markdown fences
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                if let Ok(action) = serde_json::from_str::<SubagentAction>(&trimmed[start..=end]) {
                    return Ok(action);
                }
            }
        }
    }

    Err(format!("Failed to parse subagent action: {}", content))
}

/// Format subagent results for injection into main conversation
pub fn format_subagent_data_for_prompt(subagent_result: &SubagentResult) -> String {
    let mut output = String::new();

    output.push_str("=== EXTERNAL DATA GATHERED ===\n\n");
    output.push_str(&format!("Summary: {}\n\n", subagent_result.summary));
    output.push_str(&format!(
        "Data collected from {} tool call(s):\n\n",
        subagent_result.data.len()
    ));

    for (idx, result) in subagent_result.data.iter().enumerate() {
        output.push_str(&format!(
            "Source {}: {} from {}\n",
            idx + 1,
            result.tool_name,
            result.server_id
        ));
        output.push_str(&format!(
            "Data: {}\n\n",
            serde_json::to_string_pretty(&result.result).unwrap_or_default()
        ));
    }

    output.push_str("=== END EXTERNAL DATA ===\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_accepts_plain_json() {
        let content = r#"{"action_type":"finish","reasoning":"done","final_summary":"all set"}"#;
        let action = parse_action_content(content).expect("valid action");
        assert_eq!(action.action_type, "finish");
        assert_eq!(action.final_summary.as_deref(), Some("all set"));
    }

    #[test]
    fn parse_action_accepts_fenced_json() {
        let content = "```json\n{\"action_type\":\"finish\",\"reasoning\":\"done\",\"final_summary\":\"all set\"}\n```";
        let action = parse_action_content(content).expect("valid fenced action");
        assert_eq!(action.action_type, "finish");
        assert_eq!(action.final_summary.as_deref(), Some("all set"));
    }

    #[test]
    fn format_subagent_data_includes_summary_and_sources() {
        let result = SubagentResult {
            data: vec![ToolCallResult {
                server_id: "weather".to_string(),
                tool_name: "current".to_string(),
                arguments: serde_json::json!({"city":"Tokyo"}),
                result: serde_json::json!({"temp_c":21}),
            }],
            summary: "Collected one weather datapoint".to_string(),
            iterations_used: 2,
        };

        let formatted = format_subagent_data_for_prompt(&result);
        assert!(formatted.contains("Summary: Collected one weather datapoint"));
        assert!(formatted.contains("Source 1: current from weather"));
        assert!(formatted.contains("\"temp_c\": 21"));
    }
}
