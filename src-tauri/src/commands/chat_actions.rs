use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{ipc::Channel, AppHandle, Manager, State};

use crate::models::ChatMessage;
use crate::state::AppState;

#[derive(Debug, Serialize)]
struct ChatActionLog {
    ts_ms: u128,
    session_id: String,
    message_index: usize,
    action: String,
    metadata: Value,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?;
    fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    Ok(path)
}

pub fn action_log_path_from_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("chat_actions.log")
}

pub fn shares_dir_path_from_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("shares")
}

fn build_share_file_name(session_id: &str, message_index: usize) -> String {
    let ts = now_ms();
    format!("share_{}_{}_{}.md", session_id, message_index, ts)
}

pub fn log_action_to_dir(
    base_dir: &Path,
    session_id: &str,
    message_index: usize,
    action: &str,
    metadata: Value,
) -> Result<PathBuf, String> {
    let entry = ChatActionLog {
        ts_ms: now_ms(),
        session_id: session_id.to_string(),
        message_index,
        action: action.to_string(),
        metadata,
    };
    let path = action_log_path_from_dir(base_dir);
    crate::utils::append_jsonl(&path, &entry)?;
    Ok(path)
}

pub fn share_message_to_dir(
    base_dir: &Path,
    session_id: &str,
    message_index: usize,
    content: &str,
) -> Result<PathBuf, String> {
    let shares_dir = shares_dir_path_from_dir(base_dir);
    fs::create_dir_all(&shares_dir)
        .map_err(|e| format!("Failed to create shares directory: {}", e))?;

    let file_name = build_share_file_name(session_id, message_index);
    let file_path = shares_dir.join(file_name);
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write share file: {}", e))?;

    Ok(file_path)
}

async fn ensure_message_exists(
    state: &State<'_, AppState>,
    session_id: &str,
    message_index: usize,
) -> Result<ChatMessage, String> {
    state
        .orchestrator
        .get_message(session_id, message_index)
        .await
        .ok_or_else(|| "Message not found".to_string())
}

#[tauri::command]
pub async fn chat_action_like(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_index: usize,
) -> Result<(), String> {
    ensure_message_exists(&state, &session_id, message_index).await?;
    let base_dir = app_config_dir(&app)?;
    log_action_to_dir(&base_dir, &session_id, message_index, "like", Value::Null)?;
    Ok(())
}

#[tauri::command]
pub async fn chat_action_dislike(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_index: usize,
) -> Result<(), String> {
    ensure_message_exists(&state, &session_id, message_index).await?;
    let base_dir = app_config_dir(&app)?;
    log_action_to_dir(&base_dir, &session_id, message_index, "dislike", Value::Null)?;
    Ok(())
}

#[tauri::command]
pub async fn chat_action_copy(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_index: usize,
) -> Result<(), String> {
    ensure_message_exists(&state, &session_id, message_index).await?;
    let base_dir = app_config_dir(&app)?;
    log_action_to_dir(&base_dir, &session_id, message_index, "copy", Value::Null)?;
    Ok(())
}

#[tauri::command]
pub async fn chat_action_share(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_index: usize,
) -> Result<String, String> {
    let message = ensure_message_exists(&state, &session_id, message_index).await?;
    let base_dir = app_config_dir(&app)?;
    let file_path = share_message_to_dir(&base_dir, &session_id, message_index, &message.content)?;
    let file_path_string = file_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid share file path".to_string())?;
    log_action_to_dir(
        &base_dir,
        &session_id,
        message_index,
        "share",
        serde_json::json!({ "file_path": file_path_string }),
    )?;

    file_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid share file path".to_string())
}

#[tauri::command]
pub async fn chat_action_regenerate(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_index: usize,
    temperature: f32,
    max_tokens: i32,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    state
        .orchestrator
        .regenerate_at(&session_id, message_index, temperature, max_tokens, on_event)
        .await?;
    let base_dir = app_config_dir(&app)?;
    log_action_to_dir(&base_dir, &session_id, message_index, "regenerate", Value::Null)?;
    Ok(())
}
