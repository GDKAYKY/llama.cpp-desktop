use crate::state::AppState;
use tauri::{ipc::Channel, State};

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    session_id: String,
    message: String,
    temperature: f32,
    max_tokens: i32,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    state
        .orchestrator
        .process(&session_id, message, temperature, max_tokens, on_event)
        .await
}

#[tauri::command]
pub async fn clear_chat(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.orchestrator.clear_session(&session_id).await;
    Ok(())
}
