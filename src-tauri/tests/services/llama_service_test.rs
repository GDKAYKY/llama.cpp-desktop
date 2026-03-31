use crate::common;

use llama_desktop_lib::services::llama::service::LlamaCppService;
use llama_desktop_lib::services::llama::actor::ActorMessage;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_service_start() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = common::sample_llama_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::Start { respond_to, .. }) = rx.recv().await {
            let _ = respond_to.send(Ok(12345));
        }
    });

    let pid = service.start(config).await.unwrap();
    assert_eq!(pid, 12345);
}

#[tokio::test]
async fn test_service_stop_no_config() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(None);
        }
    });

    let result = service.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_stop_with_config() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = common::sample_llama_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
        if let Some(ActorMessage::Stop { respond_to, .. }) = rx.recv().await {
            let _ = respond_to.send(Ok(()));
        }
    });

    let result = service.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_is_running() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::IsRunning { respond_to, .. }) = rx.recv().await {
            let _ = respond_to.send(true);
        }
    });

    assert!(service.is_running().await);
}

#[tokio::test]
async fn test_service_get_config() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = common::sample_llama_config();
    let config_clone = config.clone();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config_clone));
        }
    });

    let result = service.get_config().await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().port, config.port);
}

#[tokio::test]
async fn test_service_get_metrics() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetMetrics { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(llama_desktop_lib::models::ServerMetrics {
                cpu_usage: 50.0,
                mem_usage: 1024,
                gpu_usage: Some(30.0),
                vram_usage: Some(40.0),
            }));
        }
    });

    let metrics = service.get_metrics().await;
    assert!(metrics.is_some());
    assert_eq!(metrics.unwrap().cpu_usage, 50.0);
}

#[tokio::test]
async fn test_service_send_chat_no_model() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(None);
        }
    });

    let result = service.send_chat_message(None, vec![], 0.7, 1.0, 40, 512).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No model running"));
}

#[tokio::test]
async fn test_service_send_chat_success() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = common::sample_llama_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
        if let Some(ActorMessage::SendChat { respond_to, .. }) = rx.recv().await {
            let (tx, rx) = mpsc::channel(2);
            tokio::spawn(async move {
                let _ = tx.send("Hello".to_string()).await;
            });
            let _ = respond_to.send(Ok(rx));
        }
    });

    let mut receiver = service.send_chat_message(None, vec![], 0.7, 1.0, 40, 512).await.unwrap();
    let chunk = receiver.recv().await;
    assert_eq!(chunk, Some("Hello".to_string()));
}

#[tokio::test]
async fn test_service_complete_chat_no_model() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(None);
        }
    });

    let result = service.complete_chat(None, vec![], 0.7, 1.0, 40, 512, None, None, None, None, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_service_complete_chat_success() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = common::sample_llama_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
        if let Some(ActorMessage::CompleteChat { respond_to, .. }) = rx.recv().await {
            let _ = respond_to.send(Ok(serde_json::json!({"response": "test"})));
        }
    });

    let result = service.complete_chat(None, vec![], 0.7, 1.0, 40, 512, None, None, None, None, None).await;
    assert!(result.is_ok());
}
