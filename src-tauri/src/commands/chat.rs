use crate::services::orchestrator::ChatOrchestrator;
use crate::state::AppState;
use serde_json::json;
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

#[tauri::command]
pub async fn generate_chat_title(
    state: tauri::State<'_, AppState>,
    first_user_message: String,
    first_assistant_message: String,
) -> Result<String, String> {
    let system_prompt = "\
You are a title generation assistant. \
Generate a concise chat title (max 8 words) based on the provided messages. \
Return ONLY the title. Do NOT use punctuation like ':' or '-' or quotation marks. \
Do NOT provide any explanations or thinking.";

    let user_prompt = format!(
        "User: {}\nAssistant: {}",
        first_user_message, first_assistant_message
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    ];

    let response = state
        .orchestrator
        .complete_chat_once(
            messages,
            0.3,
            0.9,
            40,
            64,
            Some("none".to_string()),
            Some(0),
            Some(json!({ "enable_thinking": false })),
        )
        .await?;

    let message = &response["choices"][0]["message"];

    let raw_title = message["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Chat");

    let title = raw_title
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '.' || c == ':')
        .to_string();

    Ok(title)
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
