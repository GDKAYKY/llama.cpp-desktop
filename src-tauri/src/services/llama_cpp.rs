use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    pub llama_cpp_path: String,
    pub model_path: String,
    pub port: u16,
    pub ctx_size: u32,
    pub parallel: u32,
    pub n_gpu_layers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
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

pub struct LlamaCppService {
    process: Arc<Mutex<Option<Child>>>,
    config: Arc<Mutex<Option<LlamaCppConfig>>>,
    client: reqwest::Client,
}

impl LlamaCppService {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
        }
    }

    pub async fn start(&self, config: LlamaCppConfig) -> Result<(), String> {
        // Check if already running
        let mut process = self.process.lock().await;
        if process.is_some() {
            return Err("llama-server is already running".to_string());
        }

        let model_path = PathBuf::from(&config.model_path);
        if !model_path.exists() {
            return Err(format!("Model file not found: {}", config.model_path));
        }

        let llama_server_path = PathBuf::from(&config.llama_cpp_path).join("llama-server.exe");
        if !llama_server_path.exists() {
            return Err(format!(
                "llama-server.exe not found at: {}",
                llama_server_path.display()
            ));
        }

        let child = Command::new(&llama_server_path)
            .arg("-m")
            .arg(&config.model_path)
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-c")
            .arg(config.ctx_size.to_string())
            .arg("-np")
            .arg(config.parallel.to_string())
            .arg("-ngl")
            .arg(config.n_gpu_layers.to_string())
            .spawn()
            .map_err(|e| format!("Failed to start llama-server: {}", e))?;

        *process = Some(child);
        *self.config.lock().await = Some(config);

        // Wait for server to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut process = self.process.lock().await;
        if let Some(mut child) = process.take() {
            child
                .kill()
                .map_err(|e| format!("Failed to stop llama-server: {}", e))?;
        }
        *self.config.lock().await = None;
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        self.process.lock().await.is_some()
    }

    pub async fn send_chat_message(
        &self,
        messages: Vec<ChatMessage>,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        max_tokens: i32,
    ) -> Result<String, String> {
        let config = self.config.lock().await;
        let config = config.as_ref().ok_or("llama-server not configured")?;

        let url = format!("http://localhost:{}/v1/chat/completions", config.port);

        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            temperature,
            top_p,
            top_k,
            max_tokens,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer no-key")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        chat_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No response from model".to_string())
    }

    pub async fn get_config(&self) -> Option<LlamaCppConfig> {
        self.config.lock().await.clone()
    }
}

impl Clone for LlamaCppService {
    fn clone(&self) -> Self {
        Self {
            process: Arc::clone(&self.process),
            config: Arc::clone(&self.config),
            client: reqwest::Client::new(),
        }
    }
}
