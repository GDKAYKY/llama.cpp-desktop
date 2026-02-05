use llama_desktop_lib::utils::{append_jsonl, read_json, save_json};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
}

#[test]
fn test_save_and_read_json() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.json");

    let data = TestData {
        id: 1,
        name: "test".to_string(),
    };

    save_json(&path, &data).unwrap();
    let loaded: TestData = read_json(&path).unwrap();

    assert_eq!(loaded, data);
}

#[test]
fn test_append_jsonl() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.jsonl");

    let data1 = TestData { id: 1, name: "first".to_string() };
    let data2 = TestData { id: 2, name: "second".to_string() };

    append_jsonl(&path, &data1).unwrap();
    append_jsonl(&path, &data2).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("first"));
    assert!(lines[1].contains("second"));
}

#[test]
fn test_read_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("invalid.json");

    fs::write(&path, "not valid json").unwrap();

    let result: Result<TestData, String> = read_json(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse JSON"));
}

#[test]
fn test_save_json_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("a").join("b").join("c").join("test.json");

    let data = TestData { id: 1, name: "test".to_string() };
    save_json(&path, &data).unwrap();

    assert!(path.exists());
}

#[test]
fn test_append_jsonl_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("new.jsonl");

    let data = TestData { id: 1, name: "test".to_string() };
    append_jsonl(&path, &data).unwrap();

    assert!(path.exists());
}
