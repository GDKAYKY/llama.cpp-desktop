use llama_desktop_lib::models::{McpConfig, McpServerConfig, McpTransport};
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_config() -> McpConfig {
    McpConfig {
        servers: vec![McpServerConfig {
            id: "test".to_string(),
            name: "Test Server".to_string(),
            enabled: true,
            transport: McpTransport::Stdio,
            command: Some("test-cmd".to_string()),
            args: Some(vec!["--arg".to_string()]),
            cwd: None,
            env: None,
            url: None,
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    }
}

#[test]
fn test_save_and_load_mcp_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp.json");

    let config = create_test_config();
    llama_desktop_lib::utils::save_json(&config_path, &config).unwrap();

    assert!(config_path.exists());

    let loaded: McpConfig = llama_desktop_lib::utils::read_json(&config_path).unwrap();
    assert_eq!(loaded.servers.len(), 1);
    assert_eq!(loaded.servers[0].id, "test");
    assert_eq!(loaded.servers[0].name, "Test Server");
}

#[test]
fn test_default_mcp_config() {
    let config = McpConfig::default();
    assert_eq!(config.servers.len(), 0);
}

#[test]
fn test_mcp_config_with_http_transport() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp_http.json");

    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "http-server".to_string(),
            name: "HTTP Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost:8080".to_string()),
            headers: None,
            tool_allowlist: Some(vec!["tool1".to_string()]),
            resource_allowlist: Some(vec!["res://test".to_string()]),
        }],
    };

    llama_desktop_lib::utils::save_json(&config_path, &config).unwrap();
    let loaded: McpConfig = llama_desktop_lib::utils::read_json(&config_path).unwrap();

    assert_eq!(loaded.servers[0].transport, McpTransport::HttpSse);
    assert_eq!(
        loaded.servers[0].url,
        Some("http://localhost:8080".to_string())
    );
    assert_eq!(loaded.servers[0].tool_allowlist.as_ref().unwrap().len(), 1);
}

#[test]
fn test_read_nonexistent_config() {
    let path = PathBuf::from("/nonexistent/path/mcp.json");
    let result: Result<McpConfig, String> = llama_desktop_lib::utils::read_json(&path);
    assert!(result.is_err());
}

#[test]
fn test_save_config_creates_parent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("nested").join("dir").join("mcp.json");

    let config = create_test_config();
    llama_desktop_lib::utils::save_json(&nested_path, &config).unwrap();

    assert!(nested_path.exists());
    assert!(nested_path.parent().unwrap().exists());
}

#[test]
fn test_multiple_servers_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("multi.json");

    let config = McpConfig {
        servers: vec![
            McpServerConfig {
                id: "server1".to_string(),
                name: "Server 1".to_string(),
                enabled: true,
                transport: McpTransport::Stdio,
                command: Some("cmd1".to_string()),
                args: None,
                cwd: None,
                env: None,
                url: None,
                headers: None,
                tool_allowlist: None,
                resource_allowlist: None,
            },
            McpServerConfig {
                id: "server2".to_string(),
                name: "Server 2".to_string(),
                enabled: false,
                transport: McpTransport::HttpSse,
                command: None,
                args: None,
                cwd: None,
                env: None,
                url: Some("http://localhost:9000".to_string()),
                headers: None,
                tool_allowlist: None,
                resource_allowlist: None,
            },
        ],
    };

    llama_desktop_lib::utils::save_json(&config_path, &config).unwrap();
    let loaded: McpConfig = llama_desktop_lib::utils::read_json(&config_path).unwrap();

    assert_eq!(loaded.servers.len(), 2);
    assert_eq!(loaded.servers[0].enabled, true);
    assert_eq!(loaded.servers[1].enabled, false);
}
