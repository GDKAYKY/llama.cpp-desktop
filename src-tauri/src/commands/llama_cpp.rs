use crate::models::LlamaCppConfig;
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
    let config = LlamaCppConfig {
        llama_cpp_path: binary_path,
        model_path,
        port,
        ctx_size,
        parallel: 1,
        n_gpu_layers,
    };

    let pid = state.llama_service.start(config).await?;
    Ok(pid.to_string())
}

#[command]
pub async fn stop_llama_server(state: State<'_, AppState>) -> Result<String, String> {
    state.llama_service.stop().await?;
    Ok("Server stopped".to_string())
}

#[command]
pub async fn is_server_running(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.llama_service.is_running().await)
}

#[command]
pub async fn check_server_health(state: State<'_, AppState>) -> Result<bool, String> {
    if !state.llama_service.is_running().await {
        return Ok(false);
    }

    let config = state.llama_service.get_config().await.ok_or("No config")?;
    let url = format!("http://localhost:{}/health", config.port);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await;

    match response {
        Ok(res) => Ok(res.status().is_success()),
        Err(_) => Ok(false),
    }
}
