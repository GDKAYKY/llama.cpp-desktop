use crate::models::{LlamaCppConfig, ServerMetrics};
use crate::services::llama::LlamaCppService;
use crate::state::AppState;
use tauri::command;
use tauri::State;

#[command]
pub async fn start_llama_server(
    binary_path: String,
    model_path: String,
    port: u16,
    ctx_size: u32,
    n_gpu_layers: i32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    start_llama_server_with_service(
        &state.llama_service,
        binary_path,
        model_path,
        port,
        ctx_size,
        n_gpu_layers,
    )
    .await
}

#[command]
pub async fn stop_llama_server(state: State<'_, AppState>) -> Result<String, String> {
    stop_llama_server_with_service(&state.llama_service).await
}

#[command]
pub async fn is_server_running(state: State<'_, AppState>) -> Result<bool, String> {
    is_server_running_with_service(&state.llama_service).await
}

#[command]
pub async fn check_server_health(state: State<'_, AppState>) -> Result<bool, String> {
    check_server_health_with_service(&state.llama_service).await
}

#[command]
pub async fn get_llama_config(
    state: State<'_, AppState>,
) -> Result<Option<LlamaCppConfig>, String> {
    get_llama_config_with_service(&state.llama_service).await
}

#[command]
pub async fn get_server_metrics(
    state: State<'_, AppState>,
) -> Result<Option<ServerMetrics>, String> {
    get_server_metrics_with_service(&state.llama_service).await
}

pub async fn start_llama_server_with_service(
    service: &LlamaCppService,
    binary_path: String,
    model_path: String,
    port: u16,
    ctx_size: u32,
    n_gpu_layers: i32,
) -> Result<String, String> {
    let config = LlamaCppConfig {
        llama_cpp_path: binary_path,
        model_path,
        port,
        ctx_size,
        parallel: 1,
        n_gpu_layers,
    };

    let pid = service.start(config).await?;
    Ok(pid.to_string())
}

pub async fn stop_llama_server_with_service(service: &LlamaCppService) -> Result<String, String> {
    service.stop().await?;
    Ok("Server stopped".to_string())
}

pub async fn is_server_running_with_service(service: &LlamaCppService) -> Result<bool, String> {
    Ok(service.is_running().await)
}

pub async fn check_server_health_with_service(service: &LlamaCppService) -> Result<bool, String> {
    if !service.is_running().await {
        return Ok(false);
    }

    let config = service.get_config().await.ok_or_else(|| "No config".to_string())?;
    let url = format!("http://localhost:{}/health", config.port);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await;

    match response {
        Ok(res) => Ok(res.status().is_success()),
        Err(_) => Ok(false),
    }
}

pub async fn get_llama_config_with_service(
    service: &LlamaCppService,
) -> Result<Option<LlamaCppConfig>, String> {
    Ok(service.get_config().await)
}

pub async fn get_server_metrics_with_service(
    service: &LlamaCppService,
) -> Result<Option<ServerMetrics>, String> {
    Ok(service.get_metrics().await)
}
