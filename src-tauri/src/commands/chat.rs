use crate::services::orchestrator::ChatOrchestrator;
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
    send_message_with_orchestrator(
        &state.orchestrator,
        session_id,
        message,
        temperature,
        max_tokens,
        on_event,
    )
    .await
}

#[tauri::command]
pub async fn clear_chat(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    clear_chat_with_orchestrator(&state.orchestrator, session_id).await
}

use crate::models::ChatMessage;

#[tauri::command]
pub async fn load_history_context(
    state: State<'_, AppState>,
    session_id: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    load_history_context_with_orchestrator(&state.orchestrator, session_id, messages).await
}

pub async fn send_message_with_orchestrator(
    orchestrator: &ChatOrchestrator,
    session_id: String,
    message: String,
    temperature: f32,
    max_tokens: i32,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    orchestrator
        .process(&session_id, message, temperature, max_tokens, on_event)
        .await
}

pub async fn clear_chat_with_orchestrator(
    orchestrator: &ChatOrchestrator,
    session_id: String,
) -> Result<(), String> {
    orchestrator.clear_session(&session_id).await;
    Ok(())
}

pub async fn load_history_context_with_orchestrator(
    orchestrator: &ChatOrchestrator,
    session_id: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    orchestrator
        .set_session_history(&session_id, messages)
        .await;
    Ok(())
}
