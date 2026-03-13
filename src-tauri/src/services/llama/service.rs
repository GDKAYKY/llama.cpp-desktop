use super::actor::{ActorMessage, LlamaActor};
use crate::infrastructure::llama::process::LlamaProcessManager;
use crate::infrastructure::metrics::SystemMetricsProvider;
use crate::models::{
    ChatMessage, ChatRequest, LlamaCppConfig, ModelId, ModelLibrary, ServerMetrics,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct LlamaCppService {
    sender: mpsc::Sender<ActorMessage>,
}

impl LlamaCppService {
    pub fn new(models_path: PathBuf) -> Self {
        let registry_path = models_path.join("modelLibrary.json");
        let initial_registry = if registry_path.exists() {
            std::fs::File::open(&registry_path)
                .ok()
                .and_then(|file| {
                    let reader = std::io::BufReader::new(file);
                    serde_json::from_reader::<_, ModelLibrary>(reader).ok()
                })
                .map(|lib| {
                    lib.models
                        .into_iter()
                        .map(|m| (ModelId(m.full_identifier.clone()), m))
                        .collect::<std::collections::HashMap<ModelId, crate::models::ModelInfo>>()
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };

        let (tx, rx) = mpsc::channel(64);
        let tx_clone = tx.clone();
        let process_manager = Arc::new(LlamaProcessManager::new());
        let metrics_provider = Arc::new(SystemMetricsProvider::new());
        let mut actor =
            LlamaActor::new(rx, tx_clone, initial_registry, process_manager, metrics_provider);

        tauri::async_runtime::spawn(async move {
            actor.run().await;
        });

        Self { sender: tx }
    }

    pub fn from_sender(sender: mpsc::Sender<ActorMessage>) -> Self {
        Self { sender }
    }

    pub async fn start(&self, config: LlamaCppConfig) -> Result<u32, String> {
        let id = ModelId(config.model_path.clone());
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(ActorMessage::Start {
                model_id: id,
                config,
                respond_to: tx,
            })
            .await
            .map_err(|e| e.to_string())?;

        rx.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn stop(&self) -> Result<(), String> {
        if let Some(config) = self.get_config().await {
            let id = ModelId(config.model_path);
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ActorMessage::Stop {
                    model_id: id,
                    respond_to: tx,
                })
                .await
                .map_err(|e| e.to_string())?;
            return rx.await.map_err(|_| "Actor dropped".to_string())?;
        }
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::IsRunning {
                model_id: None,
                respond_to: tx,
            })
            .await;
        rx.await.unwrap_or(false)
    }

    pub async fn send_chat_message(
        &self,
        session_id: Option<String>,
        messages: Vec<ChatMessage>,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        max_tokens: i32,
    ) -> Result<mpsc::Receiver<String>, String> {
        let config = self.get_config().await.ok_or("No model running")?;
        let id = ModelId(config.model_path);
        let request = ChatRequest {
            model: "unknown".to_string(),
            session_id,
            messages,
            temperature,
            top_p,
            top_k,
            max_tokens,
            tools: None,
            tool_choice: None,
            stream: true,
        };
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(ActorMessage::SendChat {
                model_id: id,
                request,
                respond_to: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn get_config(&self) -> Option<LlamaCppConfig> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::GetConfig { respond_to: tx })
            .await;
        rx.await.unwrap_or(None)
    }

    pub async fn get_metrics(&self) -> Option<ServerMetrics> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::GetMetrics { respond_to: tx })
            .await;
        rx.await.unwrap_or(None)
    }

    pub async fn complete_chat(
        &self,
        session_id: Option<String>,
        messages: Vec<ChatMessage>,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        max_tokens: i32,
        tools: Option<Vec<serde_json::Value>>,
        tool_choice: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let config = self.get_config().await.ok_or("No model running")?;
        let id = ModelId(config.model_path);
        let request = ChatRequest {
            model: "unknown".to_string(),
            session_id,
            messages,
            temperature,
            top_p,
            top_k,
            max_tokens,
            tools,
            tool_choice,
            stream: false,
        };
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(ActorMessage::CompleteChat {
                model_id: id,
                request,
                respond_to: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|_| "Actor dropped".to_string())?
    }
}
