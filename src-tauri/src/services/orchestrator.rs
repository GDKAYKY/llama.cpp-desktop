use crate::services::llama_cpp::{ChatMessage, LlamaCppService};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ChatOrchestrator {
    sessions: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
    service: LlamaCppService,
}

impl ChatOrchestrator {
    pub fn new(service: LlamaCppService) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            service,
        }
    }

    pub async fn process(
        &self,
        session_id: &str,
        user_input: String,
        temperature: f32,
        max_tokens: i32,
        on_event: Channel<serde_json::Value>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;

        let history = sessions
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);

        history.push(ChatMessage {
            role: "user".to_string(),
            content: user_input,
        });

        let messages = history.clone();
        drop(sessions);

        // Streaming with user defined parameters
        let mut rx = self
            .service
            .send_chat_message(
                Some(session_id.to_string()),
                messages,
                temperature,
                0.95,
                40,
                max_tokens,
            )
            .await?;

        let mut full_response = String::new();

        while let Some(chunk) = rx.recv().await {
            full_response.push_str(&chunk);
            // println!("[Orchestrator] Sending chunk: {:?}", chunk); // Uncomment for verbose logging
            let _ = on_event.send(serde_json::json!({
                "chunk": chunk
            }));
        }

        println!("[Orchestrator] Stream finished, sending done status");
        let _ = on_event.send(serde_json::json!({
            "status": "done"
        }));

        let mut sessions = self.sessions.lock().await;
        if let Some(history) = sessions.get_mut(session_id) {
            history.push(ChatMessage {
                role: "assistant".to_string(),
                content: full_response,
            });
        }

        Ok(())
    }

    pub async fn clear_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(session_id);
    }
}
