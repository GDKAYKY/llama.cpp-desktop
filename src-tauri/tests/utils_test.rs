use llama_desktop_lib::utils::{append_jsonl, read_json, save_json};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Payload {
    name: String,
    value: u32,
}

#[test]
fn save_and_read_json_round_trip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");
    let payload = Payload {
        name: "alpha".to_string(),
        value: 42,
    };

    save_json(&path, &payload).expect("save json");
    let loaded: Payload = read_json(&path).expect("read json");
    assert_eq!(loaded, payload);
}

#[test]
fn read_json_returns_error_for_missing_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.json");
    let err = read_json::<Payload>(&path).expect_err("expected error");
    assert!(err.contains("Failed to read file"));
}

#[test]
fn read_json_returns_error_for_invalid_json() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("bad.json");
    fs::write(&path, "{not-json").expect("write");
    let err = read_json::<Payload>(&path).expect_err("expected error");
    assert!(err.contains("Failed to parse JSON"));
}

#[test]
fn append_jsonl_writes_multiple_lines() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("events.jsonl");
    let first = Payload {
        name: "first".to_string(),
        value: 1,
    };
    let second = Payload {
        name: "second".to_string(),
        value: 2,
    };

    append_jsonl(&path, &first).expect("append first");
    append_jsonl(&path, &second).expect("append second");

    let contents = fs::read_to_string(&path).expect("read jsonl");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);
    let first_loaded: Payload = serde_json::from_str(lines[0]).expect("parse first");
    let second_loaded: Payload = serde_json::from_str(lines[1]).expect("parse second");
    assert_eq!(first_loaded, first);
    assert_eq!(second_loaded, second);
}

#[test]
fn save_json_handles_paths_without_parent() {
    let payload = Payload {
        name: "alpha".to_string(),
        value: 1,
    };
    let err = save_json(Path::new(""), &payload).expect_err("expected error");
    assert!(err.contains("Failed to write"));
}

#[test]
fn append_jsonl_handles_paths_without_parent() {
    let payload = Payload {
        name: "alpha".to_string(),
        value: 1,
    };
    let err = append_jsonl(Path::new(""), &payload).expect_err("expected error");
    assert!(err.contains("Failed to open file"));
}
