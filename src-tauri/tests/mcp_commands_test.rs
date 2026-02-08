use llama_desktop_lib::commands::mcp::{
    mcp_add_server_with_service, mcp_list_servers_with_service, mcp_remove_server_with_service,
    mcp_status_with_service, mcp_tools_list_with_service, persist_config_to_path,
};
use llama_desktop_lib::models::{McpConfig, McpServerConfig, McpTransport};
use llama_desktop_lib::services::mcp::McpService;
use tempfile::tempdir;

fn sample_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
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
    }
}

#[tokio::test]
async fn mcp_list_servers_with_service_returns_configured_servers() {
    let config = McpConfig {
        servers: vec![sample_server("one")],
    };
    let service = McpService::new(config);

    let servers = mcp_list_servers_with_service(&service).await;
    assert_eq!(servers.len(), 1);
}

#[tokio::test]
async fn mcp_add_and_remove_server_with_service_updates_state() {
    let service = McpService::new(McpConfig::default());
    mcp_add_server_with_service(&service, sample_server("one"))
        .await
        .expect("add");
    let servers = mcp_list_servers_with_service(&service).await;
    assert_eq!(servers.len(), 1);

    mcp_remove_server_with_service(&service, "one".to_string())
        .await
        .expect("remove");
    let servers = mcp_list_servers_with_service(&service).await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn mcp_status_with_service_returns_statuses() {
    let config = McpConfig {
        servers: vec![sample_server("one")],
    };
    let service = McpService::new(config);
    let statuses = mcp_status_with_service(&service, None).await;
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].id, "one");
}

#[tokio::test]
async fn mcp_tools_list_with_service_errors_for_missing_server() {
    let service = McpService::new(McpConfig::default());
    let err = mcp_tools_list_with_service(&service, "missing".to_string())
        .await
        .expect_err("missing");
    assert!(err.contains("Server not found"));
}

#[test]
fn persist_config_to_path_writes_config() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("mcp.json");
    let config = McpConfig {
        servers: vec![sample_server("one")],
    };

    persist_config_to_path(&path, &config).expect("save");
    let loaded: McpConfig = llama_desktop_lib::utils::read_json(&path).expect("load");
    assert_eq!(loaded.servers.len(), 1);
}
