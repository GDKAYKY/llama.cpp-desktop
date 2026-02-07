use llama_desktop_lib::models::{LlamaCppConfig, ModelLibrary, ServerMetrics};
use llama_desktop_lib::services::llama::actor::ActorMessage;
use llama_desktop_lib::services::llama::service::LlamaCppService;
use llama_desktop_lib::utils;
use tokio::sync::mpsc;

fn sample_config() -> LlamaCppConfig {
    LlamaCppConfig {
        llama_cpp_path: "llama".to_string(),
        model_path: "model".to_string(),
        port: 8081,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    }
}

#[tokio::test]
async fn start_sends_start_and_returns_pid() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::Start { respond_to, .. }) = rx.recv().await {
            let _ = respond_to.send(Ok(123));
        }
    });

    let pid = service.start(sample_config()).await.expect("start");
    assert_eq!(pid, 123);
}

#[tokio::test]
async fn stop_returns_ok_when_no_config() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(None);
        }
    });

    service.stop().await.expect("stop");
}

#[tokio::test]
async fn stop_sends_stop_when_config_exists() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = sample_config();
    let model_path = config.model_path.clone();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
        if let Some(ActorMessage::Stop { model_id, respond_to }) = rx.recv().await {
            assert_eq!(model_id.0, model_path);
            let _ = respond_to.send(Ok(()));
        }
    });

    service.stop().await.expect("stop");
}

#[tokio::test]
async fn is_running_uses_actor_response() {
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
async fn get_config_returns_value() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = sample_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
    });

    assert!(service.get_config().await.is_some());
}

#[tokio::test]
async fn get_metrics_returns_value() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let metrics = ServerMetrics {
        cpu_usage: 1.0,
        mem_usage: 2,
        gpu_usage: None,
        vram_usage: None,
    };

    tokio::spawn(async move {
        if let Some(ActorMessage::GetMetrics { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(metrics));
        }
    });

    assert!(service.get_metrics().await.is_some());
}

#[tokio::test]
async fn send_chat_message_errors_without_config() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(None);
        }
    });

    let err = service
        .send_chat_message(None, vec![], 0.5, 1.0, 1, 1)
        .await
        .expect_err("expected error");
    assert!(err.contains("No model running"));
}

#[tokio::test]
async fn send_chat_message_returns_receiver() {
    let (tx, mut rx) = mpsc::channel(4);
    let service = LlamaCppService::from_sender(tx);
    let config = sample_config();

    tokio::spawn(async move {
        if let Some(ActorMessage::GetConfig { respond_to }) = rx.recv().await {
            let _ = respond_to.send(Some(config));
        }
        if let Some(ActorMessage::SendChat { respond_to, .. }) = rx.recv().await {
            let (tx, rx) = mpsc::channel(2);
            let _ = tx.send("hi".to_string()).await;
            let _ = respond_to.send(Ok(rx));
        }
    });

    let mut rx = service
        .send_chat_message(None, vec![], 0.5, 1.0, 1, 1)
        .await
        .expect("receiver");
    let chunk = rx.recv().await.expect("chunk");
    assert_eq!(chunk, "hi");
}

#[tokio::test]
async fn new_loads_registry_when_file_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let registry_path = dir.path().join("modelLibrary.json");
    let manifest = llama_desktop_lib::models::ModelManifest {
        schema_version: 2,
        media_type: "application/vnd.ollama.manifest.v1+json".to_string(),
        config: llama_desktop_lib::models::ManifestConfig {
            media_type: "application/vnd.ollama.image.config".to_string(),
            digest: "sha256:config".to_string(),
            size: 1,
        },
        layers: vec![llama_desktop_lib::models::ManifestLayer {
            media_type: "application/vnd.ollama.image.model".to_string(),
            digest: "sha256:model".to_string(),
            size: 1,
        }],
    };
    let library = ModelLibrary {
        models: vec![llama_desktop_lib::models::ModelInfo {
            provider: "provider".to_string(),
            library: "lib".to_string(),
            name: "name".to_string(),
            version: "v1".to_string(),
            manifest,
            model_file_path: Some("path".to_string()),
            full_identifier: "provider:name:v1".to_string(),
        }],
    };
    utils::save_json(&registry_path, &library).expect("save registry");

    let _service = LlamaCppService::new(dir.path().to_path_buf());
}

#[tokio::test]
async fn new_ignores_invalid_registry_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let registry_path = dir.path().join("modelLibrary.json");
    std::fs::write(&registry_path, "{invalid").expect("write invalid");
    let _service = LlamaCppService::new(dir.path().to_path_buf());
}
