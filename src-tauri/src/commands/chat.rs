#[tauri::command]
pub async fn send_message(message: String) -> Result<String, String> {
    Ok(format!("Echo from Rust: {}", message))
}
