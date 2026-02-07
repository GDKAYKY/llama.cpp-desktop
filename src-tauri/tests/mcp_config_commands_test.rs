use llama_desktop_lib::commands::mcp_config::{
    build_mcp_config_path_from_dir, load_mcp_config_from_path, reset_mcp_config_at_path,
    save_mcp_config_to_path,
};
use llama_desktop_lib::models::{McpConfig, McpServerConfig, McpTransport};
use tempfile::tempdir;

fn sample_config() -> McpConfig {
    McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    }
}

#[test]
fn build_mcp_config_path_from_dir_appends_filename() {
    let dir = tempdir().expect("tempdir");
    let path = build_mcp_config_path_from_dir(dir.path().to_path_buf());
    assert!(path.ends_with("mcp.json"));
}

#[test]
fn load_mcp_config_from_path_returns_default_when_missing() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("mcp.json");
    let config = load_mcp_config_from_path(&path).expect("default");
    assert!(config.servers.is_empty());
}

#[test]
fn save_and_load_mcp_config_round_trip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("mcp.json");
    let config = sample_config();
    save_mcp_config_to_path(&path, &config).expect("save");

    let loaded = load_mcp_config_from_path(&path).expect("load");
    assert_eq!(loaded.servers.len(), 1);
    assert_eq!(loaded.servers[0].id, "one");
}

#[test]
fn reset_mcp_config_at_path_writes_default() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("mcp.json");

    let reset = reset_mcp_config_at_path(&path).expect("reset");
    assert!(reset.servers.is_empty());
    let loaded = load_mcp_config_from_path(&path).expect("load");
    assert!(loaded.servers.is_empty());
}
