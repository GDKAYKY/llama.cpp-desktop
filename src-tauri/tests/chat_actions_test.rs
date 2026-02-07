use llama_desktop_lib::commands::chat_actions::{
    action_log_path_from_dir, log_action_to_dir, share_message_to_dir, shares_dir_path_from_dir,
};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn action_log_path_is_deterministic() {
    let dir = Path::new("/tmp/example");
    let path = action_log_path_from_dir(dir);
    assert!(path.ends_with("chat_actions.log"));
}

#[test]
fn shares_dir_path_is_deterministic() {
    let dir = Path::new("/tmp/example");
    let path = shares_dir_path_from_dir(dir);
    assert!(path.ends_with("shares"));
}

#[test]
fn log_action_to_dir_writes_jsonl_entry() {
    let dir = tempdir().expect("tempdir");
    let path = log_action_to_dir(
        dir.path(),
        "session-1",
        2,
        "like",
        serde_json::json!({ "note": "ok" }),
    )
    .expect("log action");

    let contents = std::fs::read_to_string(&path).expect("read log");
    let entry: serde_json::Value =
        serde_json::from_str(contents.lines().next().unwrap()).expect("parse");
    assert_eq!(entry["session_id"], "session-1");
    assert_eq!(entry["message_index"], 2);
    assert_eq!(entry["action"], "like");
}

#[test]
fn share_message_to_dir_creates_file() {
    let dir = tempdir().expect("tempdir");
    let file_path = share_message_to_dir(dir.path(), "session-2", 4, "hello")
        .expect("share");
    let content = std::fs::read_to_string(&file_path).expect("read share");
    assert_eq!(content, "hello");
}
