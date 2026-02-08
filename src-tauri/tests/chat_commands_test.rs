use llama_desktop_lib::commands::chat::{
    clear_chat_with_orchestrator, load_history_context_with_orchestrator,
    send_message_with_orchestrator,
};
use llama_desktop_lib::models::ChatMessage;
use llama_desktop_lib::services::llama::{ActorMessage, LlamaCppService};
use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use tauri::ipc::Channel;
use tauri::ipc::InvokeResponseBody;
use tokio::sync::mpsc;

fn sample_config() -> llama_desktop_lib::models::LlamaCppConfig {
    llama_desktop_lib::models::LlamaCppConfig {
        llama_cpp_path: "llama".to_string(),
        model_path: "model".to_string(),
        port: 8080,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    }
}

fn test_channel() -> (
    Channel<serde_json::Value>,
    mpsc::Receiver<serde_json::Value>,
) {
    let (tx, rx) = mpsc::channel::<serde_json::Value>(8);

    let channel = Channel::new(move |payload: InvokeResponseBody| -> tauri::Result<()> {
        // Converte InvokeResponseBody em serde_json::Value
        let value: serde_json::Value = match payload {
            InvokeResponseBody::Json(text) => {
                serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text))
            }
            InvokeResponseBody::Raw(bytes) => serde_json::from_slice(&bytes).unwrap_or(
                serde_json::Value::String(String::from_utf8_lossy(&bytes).to_string()),
            ),
        };

        // Envia pro canal
        let _ = tx.try_send(value);
        Ok(())
    });

    (channel, rx)
}

fn mock_service(chunks: Vec<&'static str>) -> LlamaCppService {
    let (tx, mut rx) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let (out_tx, out_rx) = mpsc::channel(8);
                    for chunk in &chunks {
                        let _ = out_tx.send(chunk.to_string()).await;
                    }
                    drop(out_tx);
                    let _ = respond_to.send(Ok(out_rx));
                }
                _ => {}
            }
        }
    });
    LlamaCppService::from_sender(tx)
}

#[tokio::test]
async fn send_message_with_orchestrator_streams_and_records() {
    let service = mock_service(vec!["Hello", " world"]);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, mut rx) = test_channel();

    send_message_with_orchestrator(
        &orchestrator,
        "session-1".to_string(),
        "Hi".to_string(),
        0.5,
        10,
        channel,
    )
    .await
    .expect("send");

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    assert!(events.iter().any(|e| e.get("chunk").is_some()));

    let message = orchestrator.get_message("session-1", 1).await;
    assert_eq!(message.map(|m| m.content), Some("Hello world".to_string()));
}

#[tokio::test]
async fn clear_chat_with_orchestrator_removes_history() {
    let service = mock_service(vec!["ok"]);
    let orchestrator = ChatOrchestrator::new(service);
    orchestrator
        .set_session_history(
            "session-1",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
        )
        .await;

    clear_chat_with_orchestrator(&orchestrator, "session-1".to_string())
        .await
        .expect("clear");
    let message = orchestrator.get_message("session-1", 0).await;
    assert!(message.is_none());
}

#[tokio::test]
async fn load_history_context_with_orchestrator_sets_history() {
    let service = mock_service(vec!["ok"]);
    let orchestrator = ChatOrchestrator::new(service);
    let history = vec![ChatMessage {
        role: "user".to_string(),
        content: "hello".to_string(),
    }];

    load_history_context_with_orchestrator(&orchestrator, "session-1".to_string(), history.clone())
        .await
        .expect("load");

    let message = orchestrator.get_message("session-1", 0).await;
    assert_eq!(
        message.as_ref().map(|m| &m.content),
        Some(&history[0].content)
    );
}
