use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn send_message(message: String, state: State<'_, AppState>) -> Result<String, String> {
    state
        .llama_service
        .send_chat_message(
            vec![crate::services::llama_cpp::ChatMessage {
                role: "user".to_string(),
                content: message,
            }],
            0.7,
            0.95,
            40,
            512,
        )
        .await
}
