use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub session_id: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_budget_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_forced_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    pub usage: serde_json::Value,
}

/// What the LLM returns during intent classification.
/// The host uses this to resolve tools deterministically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub needs_external: bool,
    pub query: String,
    /// Optional: LLM may suggest a tool, but host validates.
    pub suggested_tool: Option<String>,
    /// Optional: LLM may suggest a server, but host validates.
    pub suggested_server: Option<String>,
    /// Structured arguments for the tool.
    pub arguments: Option<serde_json::Value>,
    #[serde(default)]
    pub needs_multi_step: bool,
    #[serde(default)]
    pub multi_step_reasoning: Option<String>,
}
