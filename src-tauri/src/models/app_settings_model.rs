use serde::{Deserialize, Serialize};
use crate::models::GpuLayerStrategy;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub models_directory: Option<String>,
    pub llama_directory: Option<String>,
    pub theme: String,
    pub language: String,
    pub max_tokens: u32,
    pub context_size: u32,
    pub temperature: f32,
    pub auto_save_chat: bool,
    pub chat_history_limit: u32,
    pub server_port: u16,
    pub llama_server_parallel: u32,
    pub llama_server_gpu_layers: i32,
    pub llama_server_gpu_layer_strategy: GpuLayerStrategy,
    pub llama_server_gpu_device: String,
    pub llama_server_jinja: bool,
    pub llama_server_extra_args: Vec<String>,
    pub web_search_provider: String,
    pub web_search_mcp_id: Option<String>,
    pub chat_header_style: String,
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
            models_directory: None,
            llama_directory: None,
            theme: "dark".to_string(),
            language: "en".to_string(),
            max_tokens: 2048,
            context_size: 8192,
            temperature: 0.7,
            auto_save_chat: true,
            chat_history_limit: 50,
            server_port: 8080,
            llama_server_parallel: 1,
            llama_server_gpu_layers: 33,
            llama_server_gpu_layer_strategy: GpuLayerStrategy::Manual,
            llama_server_gpu_device: "0".to_string(),
            llama_server_jinja: true,
            llama_server_extra_args: Vec::new(),
            web_search_provider: "tavily".to_string(),
            web_search_mcp_id: None,
            chat_header_style: "default".to_string(),
        }
    }
}
