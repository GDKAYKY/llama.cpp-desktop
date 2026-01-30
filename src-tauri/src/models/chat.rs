use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
