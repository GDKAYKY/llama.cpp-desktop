use llama_desktop_lib::commands::chat_actions::{
    log_action_to_dir, share_message_to_dir,
};
use llama_desktop_lib::models::ChatMessage;
use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn chat_action_share_creates_file_and_returns_path() {
    let temp = tempdir().expect("tempdir");
    let base_dir = temp.path();
    let session_id = "session-1";
    let message_index = 2;
    let content = "hello share";

    let path =
        share_message_to_dir(base_dir, session_id, message_index, content).expect("share");

    assert!(path.exists(), "share file should exist");
    let saved = std::fs::read_to_string(&path).expect("read share file");
    assert_eq!(saved, content);
    assert!(path.to_string_lossy().contains("shares"));
}

#[test]
fn chat_action_like_writes_valid_jsonl() {
    let temp = tempdir().expect("tempdir");
    let base_dir = temp.path();

    let log_path = log_action_to_dir(
        base_dir,
        "session-1",
        0,
        "like",
        Value::Null,
    )
    .expect("log action");

    let content = std::fs::read_to_string(&log_path).expect("read log");
    let line = content.lines().next().expect("log line");
    let json: serde_json::Value = serde_json::from_str(line).expect("valid json");
    assert_eq!(json["action"], "like");
    assert_eq!(json["session_id"], "session-1");
    assert_eq!(json["message_index"], 0);
}

#[test]
fn regenerate_at_fails_on_invalid_index() {
    let history = vec![ChatMessage {
        role: "assistant".to_string(),
        content: "hi".to_string(),
    }];

    let err = ChatOrchestrator::prepare_regenerate_history(&history, 5)
        .expect_err("should fail");
    assert!(err.contains("Message not found"));
}

#[test]
fn regenerate_at_fails_on_non_assistant_message() {
    let history = vec![ChatMessage {
        role: "user".to_string(),
        content: "hi".to_string(),
    }];

    let err = ChatOrchestrator::prepare_regenerate_history(&history, 0)
        .expect_err("should fail");
    assert!(err.contains("assistant"));
}
