use llama_desktop_lib::commands::llama_cpp::{
    check_server_health_with_service, get_llama_config_with_service,
    get_server_metrics_with_service, is_server_running_with_service,
    start_llama_server_with_service, stop_llama_server_with_service,
};
use llama_desktop_lib::models::{LlamaCppConfig, ServerMetrics};
use llama_desktop_lib::services::llama::{ActorMessage, LlamaCppService};
use tokio::sync::mpsc;

fn sample_config() -> LlamaCppConfig {
    LlamaCppConfig {
        llama_cpp_path: "llama".to_string(),
        model_path: "model".to_string(),
        port: 8080,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    }
}

fn sample_metrics() -> ServerMetrics {
    ServerMetrics {
        cpu_usage: 1.2,
        mem_usage: 2048,
        gpu_usage: Some(0.4),
        vram_usage: Some(0.2),
    }
}

fn mock_service(running: bool) -> LlamaCppService {
    let (tx, mut rx) = mpsc::channel(8);
    let config = sample_config();
    let metrics = sample_metrics();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::Start { respond_to, .. } => {
                    let _ = respond_to.send(Ok(123));
                }
                ActorMessage::Stop { respond_to, .. } => {
                    let _ = respond_to.send(Ok(()));
                }
                ActorMessage::IsRunning { respond_to, .. } => {
                    let _ = respond_to.send(running);
                }
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(config.clone()));
                }
                ActorMessage::GetMetrics { respond_to } => {
                    let _ = respond_to.send(Some(metrics.clone()));
                }
                _ => {}
            }
        }
    });
    LlamaCppService::from_sender(tx)
}

#[tokio::test]
async fn start_stop_and_status_helpers_work() {
    let service = mock_service(false);

    let pid = start_llama_server_with_service(
        &service,
        "bin".to_string(),
        "model".to_string(),
        8080,
        128,
        0,
    )
    .await
    .expect("start");
    assert_eq!(pid, "123");

    let stopped = stop_llama_server_with_service(&service)
        .await
        .expect("stop");
    assert_eq!(stopped, "Server stopped");

    let running = is_server_running_with_service(&service)
        .await
        .expect("status");
    assert!(!running);
}

#[tokio::test]
async fn config_and_metrics_helpers_return_values() {
    let service = mock_service(false);

    let config = get_llama_config_with_service(&service)
        .await
        .expect("config")
        .expect("config should be some");
    let expected_config = sample_config();
    assert_eq!(config.llama_cpp_path, expected_config.llama_cpp_path);
    assert_eq!(config.model_path, expected_config.model_path);
    assert_eq!(config.port, expected_config.port);

    let metrics = get_server_metrics_with_service(&service)
        .await
        .expect("metrics")
        .expect("metrics should be some");
    let expected_metrics = sample_metrics();
    assert_eq!(metrics.cpu_usage, expected_metrics.cpu_usage);
    assert_eq!(metrics.mem_usage, expected_metrics.mem_usage);
}

#[tokio::test]
async fn check_server_health_returns_false_when_not_running() {
    let service = mock_service(false);
    let healthy = check_server_health_with_service(&service)
        .await
        .expect("health");
    assert!(!healthy);
}
