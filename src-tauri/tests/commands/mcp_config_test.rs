use crate::common;

use llama_desktop_lib::commands::mcp_config::{
    build_mcp_config_path_from_dir, build_default_mcp_config_path_from_dir,
    load_mcp_config_from_path, save_mcp_config_to_path, reset_mcp_config_at_path
};
use llama_desktop_lib::models::McpConfig;

#[test]
fn test_build_mcp_config_path_from_dir() {
    let dir = common::temp_dir();
    let path = build_mcp_config_path_from_dir(dir.path().to_path_buf());
    
    assert!(path.to_str().unwrap().ends_with("mcp.json"));
}

#[test]
fn test_build_default_mcp_config_path_from_dir() {
    let dir = common::temp_dir();
    let path = build_default_mcp_config_path_from_dir(dir.path().to_path_buf());
    
    assert!(path.to_str().unwrap().ends_with("defaultMcp.json"));
}

#[test]
fn test_load_mcp_config_nonexistent() {
    let dir = common::temp_dir();
    let path = dir.path().join("mcp.json");
    
    let result = load_mcp_config_from_path(&path);
    assert!(result.is_err());
}

#[test]
fn test_save_and_load_mcp_config() {
    let dir = common::temp_dir();
    let path = dir.path().join("mcp.json");
    
    let mut config = McpConfig::default();
    config.servers.push(common::sample_mcp_server("test1"));
    
    save_mcp_config_to_path(&path, &config).unwrap();
    let loaded = load_mcp_config_from_path(&path).unwrap();
    
    assert_eq!(loaded.servers.len(), 1);
    assert_eq!(loaded.servers[0].id, "test1");
}

#[test]
fn test_reset_mcp_config() {
    let dir = common::temp_dir();
    let path = dir.path().join("mcp.json");
    
    let mut config = McpConfig::default();
    config.servers.push(common::sample_mcp_server("test1"));
    save_mcp_config_to_path(&path, &config).unwrap();
    
    let reset = reset_mcp_config_at_path(&path).unwrap();
    assert!(reset.servers.is_empty());
    
    let loaded = load_mcp_config_from_path(&path).unwrap();
    assert!(loaded.servers.is_empty());
}

#[test]
fn test_save_mcp_config_creates_parent_dirs() {
    let dir = common::temp_dir();
    let path = dir.path().join("nested/config/mcp.json");
    
    let config = McpConfig::default();
    save_mcp_config_to_path(&path, &config).unwrap();
    
    assert!(path.exists());
}

#[test]
fn test_load_mcp_config_invalid_json() {
    let dir = common::temp_dir();
    let path = dir.path().join("mcp.json");
    std::fs::write(&path, "{invalid json}").unwrap();
    
    let result = load_mcp_config_from_path(&path);
    assert!(result.is_err());
}
