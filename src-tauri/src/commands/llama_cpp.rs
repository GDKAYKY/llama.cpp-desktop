use crate::models::{LlamaCppConfig, ServerMetrics};
use crate::services::llama::LlamaCppService;
use crate::state::AppState;
use tauri::command;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use std::path::PathBuf;

// ─── Comando: ensure_chat_template ────────────────────────────────────────────
//
// Garante que o chat template Jinja de um repo HuggingFace esteja em cache.
// Retorna o caminho absoluto do arquivo `.jinja` local.

#[command]
pub async fn ensure_chat_template(
    app: AppHandle,
    hf_repo: String,
) -> Result<String, String> {
    let path = crate::services::templates::ensure_hf_chat_template(&app, &hf_repo, None).await?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Caminho de template inválido".to_string())
}

// ─── Comando: start_llama_server ──────────────────────────────────────────────

#[command]
pub async fn start_llama_server(
    binary_path: String,
    model_path: String,
    port: u16,
    ctx_size: u32,
    n_gpu_layers: i32,
    parallel: Option<u32>,
    chat_template: Option<String>,
    chat_template_file: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    start_llama_server_with_service(
        &state.llama_service,
        binary_path,
        model_path,
        port,
        ctx_size,
        n_gpu_layers,
        parallel,
        chat_template,
        chat_template_file,
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
    parallel: Option<u32>,
    chat_template: Option<String>,
    chat_template_file: Option<String>,
) -> Result<String, String> {
    let config = LlamaCppConfig {
        llama_cpp_path: binary_path,
        model_path,
        port,
        ctx_size,
        parallel: parallel.unwrap_or(1),
        n_gpu_layers,
        chat_template,
        chat_template_file,
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
