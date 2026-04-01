use crate::common;

use llama_desktop_lib::commands::models::{
    parse_model_path, digest_to_blob_filename, find_model_blob_path, parse_model_manifest_sync
};

#[test]
fn test_parse_model_path_valid() {
    let path = "/models/manifests/ollama/library/llama3/latest/manifest.json";
    let result = parse_model_path(path).unwrap();
    
    assert_eq!(result.0, "ollama");
    assert_eq!(result.1, "library");
    assert_eq!(result.2, "llama3");
    assert_eq!(result.3, "latest");
}

#[test]
fn test_parse_model_path_windows_style() {
    // Changed to a relative path to avoid Windows drive prefix issues
    let path = "models\\manifests\\hf\\user\\model\\v1\\manifest.json";
    let result = parse_model_path(path).unwrap();
    
    assert_eq!(result.0, "hf");
    assert_eq!(result.1, "user");
    assert_eq!(result.2, "model");
    assert_eq!(result.3, "v1");
}

#[test]
fn test_parse_model_path_no_manifests_dir() {
    let path = "/models/some/path/model.json";
    let result = parse_model_path(path);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("manifests"));
}

#[test]
fn test_parse_model_path_insufficient_components() {
    let path = "/models/manifests/provider/library";
    let result = parse_model_path(path);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid path structure"));
}

#[test]
fn test_digest_to_blob_filename() {
    assert_eq!(
        digest_to_blob_filename("sha256:abc123def456"),
        "sha256-abc123def456"
    );
    assert_eq!(
        digest_to_blob_filename("sha256:xyz"),
        "sha256-xyz"
    );
}

#[test]
fn test_find_model_blob_path_exists() {
    let dir = common::temp_dir();
    let blobs_dir = dir.path().join("blobs");
    std::fs::create_dir_all(&blobs_dir).unwrap();
    
    let blob_file = blobs_dir.join("sha256-abc123");
    std::fs::write(&blob_file, "test data").unwrap();
    
    let result = find_model_blob_path(dir.path(), "sha256:abc123");
    assert!(result.is_some());
    assert!(result.unwrap().contains("sha256-abc123"));
}

#[test]
fn test_find_model_blob_path_not_exists() {
    let dir = common::temp_dir();
    let result = find_model_blob_path(dir.path(), "sha256:nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_parse_model_manifest_sync_valid() {
    let dir = common::temp_dir();
    let manifests_dir = dir.path().join("manifests/test/lib/model/v1");
    std::fs::create_dir_all(&manifests_dir).unwrap();
    
    let manifest_path = manifests_dir.join("manifest.json");
    let manifest = common::create_test_model_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).unwrap();
    
    let metadata_dir = dir.path().join("metadata");
    std::fs::create_dir_all(&metadata_dir).unwrap();
    
    let result = parse_model_manifest_sync(
        manifest_path.to_str().unwrap().to_string(),
        dir.path().to_str().unwrap().to_string(),
        metadata_dir.to_str().unwrap(),
    );
    
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.provider, "test");
    assert_eq!(info.library, "lib");
    assert_eq!(info.name, "model");
    assert_eq!(info.version, "v1");
}

#[test]
fn test_parse_model_manifest_sync_invalid_path() {
    let result = parse_model_manifest_sync(
        "/invalid/path.json".to_string(),
        "/models".to_string(),
        "/metadata",
    );
    
    assert!(result.is_err());
}
