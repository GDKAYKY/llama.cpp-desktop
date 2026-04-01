use crate::common;

use llama_desktop_lib::utils;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
}

#[test]
fn test_save_and_read_json() {
    let dir = common::temp_dir();
    let path = dir.path().join("test.json");
    
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    
    utils::save_json(&path, &data).unwrap();
    let loaded: TestData = utils::read_json(&path).unwrap();
    
    assert_eq!(data, loaded);
}

#[test]
fn test_save_json_creates_parent_dirs() {
    let dir = common::temp_dir();
    let path = dir.path().join("nested/deep/test.json");
    
    let data = TestData {
        name: "nested".to_string(),
        value: 100,
    };
    
    utils::save_json(&path, &data).unwrap();
    assert!(path.exists());
}

#[test]
fn test_read_json_nonexistent_file() {
    let result: Result<TestData, String> = utils::read_json(Path::new("/nonexistent.json"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read file"));
}

#[test]
fn test_read_json_invalid_json() {
    let dir = common::temp_dir();
    let path = dir.path().join("invalid.json");
    std::fs::write(&path, "{invalid json}").unwrap();
    
    let result: Result<TestData, String> = utils::read_json(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse JSON"));
}

#[test]
fn test_append_jsonl() {
    let dir = common::temp_dir();
    let path = dir.path().join("data.jsonl");
    
    let data1 = TestData { name: "first".to_string(), value: 1 };
    let data2 = TestData { name: "second".to_string(), value: 2 };
    
    utils::append_jsonl(&path, &data1).unwrap();
    utils::append_jsonl(&path, &data2).unwrap();
    
    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    
    assert_eq!(lines.len(), 2);
    
    let loaded1: TestData = serde_json::from_str(lines[0]).unwrap();
    let loaded2: TestData = serde_json::from_str(lines[1]).unwrap();
    
    assert_eq!(data1, loaded1);
    assert_eq!(data2, loaded2);
}

#[test]
fn test_append_jsonl_creates_parent_dirs() {
    let dir = common::temp_dir();
    let path = dir.path().join("logs/nested/data.jsonl");
    
    let data = TestData { name: "log".to_string(), value: 99 };
    utils::append_jsonl(&path, &data).unwrap();
    
    assert!(path.exists());
}
