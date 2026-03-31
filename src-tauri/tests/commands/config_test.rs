use crate::common;

use llama_desktop_lib::commands::config::{
    build_config_file_path_from_dir, get_config_from_path, reset_config_at_path,
    save_config_to_path,
};
use llama_desktop_lib::models::AppConfig;

#[test]
fn test_build_config_file_path_from_dir() {
    let dir = common::temp_dir();
    let path = build_config_file_path_from_dir(dir.path().to_path_buf());

    assert!(path.to_str().unwrap().ends_with("config.json"));
}

#[test]
fn test_get_config_from_path_nonexistent() {
    let dir = common::temp_dir();
    let path = dir.path().join("config.json");

    let config = get_config_from_path(&path).unwrap();
    assert!(config.models_directory.is_none());
}

#[test]
fn test_save_and_load_config() {
    let dir = common::temp_dir();
    let path = dir.path().join("config.json");

    let config = AppConfig {
        models_directory: Some("/models".to_string()),
        llama_directory: Some("/bin/llama".to_string()),
        ..Default::default()
    };

    save_config_to_path(&path, &config).unwrap();
    let loaded = get_config_from_path(&path).unwrap();

    assert_eq!(loaded.models_directory, Some("/models".to_string()));
    assert_eq!(loaded.llama_directory, Some("/bin/llama".to_string()));
}

#[test]
fn test_reset_config() {
    let dir = common::temp_dir();
    let path = dir.path().join("config.json");

    let config = AppConfig {
        models_directory: Some("/models".to_string()),
        llama_directory: Some("/bin/llama".to_string()),
        ..Default::default()
    };
    save_config_to_path(&path, &config).unwrap();

    let reset = reset_config_at_path(&path).unwrap();
    assert!(reset.models_directory.is_none());

    let loaded = get_config_from_path(&path).unwrap();
    assert!(loaded.models_directory.is_none());
}

#[test]
fn test_save_config_creates_parent_dirs() {
    let dir = common::temp_dir();
    let path = dir.path().join("nested/deep/config.json");

    let config = AppConfig::default();
    save_config_to_path(&path, &config).unwrap();

    assert!(path.exists());
}
