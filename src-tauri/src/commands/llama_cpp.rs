use crate::services::llama_cpp::LlamaCppConfig;
use crate::state::AppState;
use tauri::command;
use tauri::State;

#[command]
pub async fn init_llama(
    llama_path: String,
    model_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = LlamaCppConfig {
        llama_cpp_path: llama_path,
        model_path,
        port: 8080,
        ctx_size: 4096,
        parallel: 4,
        n_gpu_layers: 33,
    };

    state.llama_service.start(config).await?;
    Ok("Model loaded".to_string())
}

#[command]
pub async fn shutdown_llama(state: State<'_, AppState>) -> Result<String, String> {
    state.llama_service.stop().await?;
    Ok("Model unloaded".to_string())
}
