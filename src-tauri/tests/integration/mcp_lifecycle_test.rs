use crate::common;

use llama_desktop_lib::services::mcp::service::McpService;
use llama_desktop_lib::models::McpConfig;

#[tokio::test]
async fn test_mcp_full_lifecycle() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    // Add server
    let mut server = common::sample_mcp_server("lifecycle_test");
    server.enabled = true;
    service.add_server(server.clone()).await.unwrap();
    
    // Verify added
    let servers = service.list_servers().await;
    assert_eq!(servers.len(), 1);
    
    // Update server
    server.name = "Updated".to_string();
    service.update_server(server).await.unwrap();
    
    let servers = service.list_servers().await;
    assert_eq!(servers[0].name, "Updated");
    
    // Remove server
    service.remove_server("lifecycle_test").await.unwrap();
    
    let servers = service.list_servers().await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn test_mcp_config_persistence() {
    let dir = common::temp_dir();
    let config_path = dir.path().join("mcp.json");
    
    let mut config = McpConfig::default();
    config.servers.push(common::sample_mcp_server("persist1"));
    config.servers.push(common::sample_mcp_server("persist2"));
    
    llama_desktop_lib::utils::save_json(&config_path, &config).unwrap();
    
    let loaded: McpConfig = llama_desktop_lib::utils::read_json(&config_path).unwrap();
    assert_eq!(loaded.servers.len(), 2);
}

#[tokio::test]
async fn test_status_multiple_servers() {
    let mut config = McpConfig::default();
    config.servers.push(common::sample_mcp_server("server1"));
    config.servers.push(common::sample_mcp_server("server2"));
    
    let service = McpService::new(config, None);
    
    let status = service.status(None).await;
    // Should return status for all servers (even if not connected)
    assert_eq!(status.len(), 2);
}
