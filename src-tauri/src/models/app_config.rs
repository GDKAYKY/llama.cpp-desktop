use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub models_directory: Option<String>,
    pub llama_directory: Option<String>,
    pub theme: String,
    pub language: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub auto_save_chat: bool,
    pub chat_history_limit: u32,
    pub server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigError {
    Io(String),
    NotFound,
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::NotFound => write!(f, "Config not found"),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            models_directory: Some("E:\\models".to_string()),
            llama_directory: Some("E:\\src\\llama_cpp".to_string()),
            theme: "dark".to_string(),
            language: "en".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            auto_save_chat: true,
            chat_history_limit: 50,
            server_port: 8080,
        }
    }
}
