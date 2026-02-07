use llama_desktop_lib::models::ChatMessage;
use llama_desktop_lib::services::llama::ActorMessage;
use llama_desktop_lib::services::llama::LlamaCppService;
use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use tauri::ipc::Channel;
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

fn test_channel() -> (Channel<serde_json::Value>, mpsc::Receiver<serde_json::Value>) {
    let (tx, rx) = mpsc::channel::<serde_json::Value>(8);
    let channel = Channel::new(move |payload| -> tauri::Result<()> {
        let value = serde_json::to_value(payload)?;
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
async fn prepare_regenerate_history_validates_role() {
    let history = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "hello".to_string(),
        },
    ];

    let err = ChatOrchestrator::prepare_regenerate_history(&history, 0)
        .expect_err("not assistant");
    assert!(err.contains("Target message is not an assistant"));

    let ok = ChatOrchestrator::prepare_regenerate_history(&history, 1).expect("ok");
    assert_eq!(ok.len(), 1);
}

#[tokio::test]
async fn prepare_regenerate_history_errors_when_missing() {
    let history = vec![ChatMessage {
        role: "assistant".to_string(),
        content: "hi".to_string(),
    }];
    let err = ChatOrchestrator::prepare_regenerate_history(&history, 2).expect_err("missing");
    assert!(err.contains("Message not found"));
}

#[tokio::test]
async fn process_appends_messages_and_updates_history() {
    let service = mock_service(vec!["Hello", " world"]);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, mut rx) = test_channel();

    orchestrator
        .process("session-1", "Hi".to_string(), 0.5, 10, channel)
        .await
        .expect("process");

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    assert!(events.iter().any(|e| e.get("chunk").is_some()));

    let message = orchestrator.get_message("session-1", 1).await;
    assert_eq!(message.map(|m| m.content), Some("Hello world".to_string()));
}

#[tokio::test]
async fn process_propagates_send_errors() {
    let (tx, mut rx) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let _ = respond_to.send(Err("boom".to_string()));
                }
                _ => {}
            }
        }
    });
    let service = LlamaCppService::from_sender(tx);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, _rx) = test_channel();

    let err = orchestrator
        .process("session-1", "Hi".to_string(), 0.5, 10, channel)
        .await
        .expect_err("error");
    assert!(err.contains("boom"));
}

#[tokio::test]
async fn regenerate_at_replaces_message() {
    let service = mock_service(vec!["New", " answer"]);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, _rx) = test_channel();
    orchestrator
        .set_session_history(
            "session-1",
            vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Ask".to_string(),
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Old".to_string(),
                },
            ],
        )
        .await;

    orchestrator
        .regenerate_at("session-1", 1, 0.5, 10, channel)
        .await
        .expect("regenerate");

    let message = orchestrator.get_message("session-1", 1).await;
    assert_eq!(message.map(|m| m.content), Some("New answer".to_string()));
}

#[tokio::test]
async fn regenerate_at_errors_when_session_missing() {
    let service = mock_service(vec!["ok"]);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, _rx) = test_channel();
    let err = orchestrator
        .regenerate_at("missing", 0, 0.5, 10, channel)
        .await
        .expect_err("error");
    assert!(err.contains("Session not found"));
}

#[tokio::test]
async fn regenerate_at_errors_when_message_removed() {
    let (tx, mut rx) = mpsc::channel(8);
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let mut gate_rx = Some(gate_rx);
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let (out_tx, out_rx) = mpsc::channel(8);
                    let _ = respond_to.send(Ok(out_rx));
                    if let Some(rx) = gate_rx.take() {
                        let _ = rx.await;
                    }
                    let _ = out_tx.send("done".to_string()).await;
                }
                _ => {}
            }
        }
    });
    let service = LlamaCppService::from_sender(tx);
    let orchestrator = ChatOrchestrator::new(service);
    let (channel, _rx) = test_channel();
    orchestrator
        .set_session_history(
            "session-1",
            vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Ask".to_string(),
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Old".to_string(),
                },
            ],
        )
        .await;

    let handle = tokio::spawn({
        let orchestrator = orchestrator.clone();
        async move { orchestrator.regenerate_at("session-1", 1, 0.5, 10, channel).await }
    });

    orchestrator
        .remove_message("session-1", 1)
        .await
        .expect("remove");
    let _ = gate_tx.send(());

    let err = handle.await.expect("handle").expect_err("error");
    assert!(err.contains("Message removed"));
}
