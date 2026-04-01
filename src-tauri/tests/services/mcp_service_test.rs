use crate::common;

use llama_desktop_lib::models::{McpConfig};
use llama_desktop_lib::services::mcp::service::McpService;

#[tokio::test]
async fn test_mcp_service_new() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let servers = service.list_servers().await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn test_mcp_service_add_server() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("test1");
    service.add_server(server.clone()).await.unwrap();
    
    let servers = service.list_servers().await;
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].id, "test1");
}

#[tokio::test]
async fn test_mcp_service_add_duplicate_server() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("test1");
    service.add_server(server.clone()).await.unwrap();
    
    let result = service.add_server(server).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[tokio::test]
async fn test_mcp_service_update_server() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let mut server = common::sample_mcp_server("test1");
    service.add_server(server.clone()).await.unwrap();
    
    server.name = "Updated Name".to_string();
    service.update_server(server).await.unwrap();
    
    let servers = service.list_servers().await;
    assert_eq!(servers[0].name, "Updated Name");
}

#[tokio::test]
async fn test_mcp_service_update_nonexistent() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("nonexistent");
    let result = service.update_server(server).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_service_remove_server() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("test1");
    service.add_server(server).await.unwrap();
    
    service.remove_server("test1").await.unwrap();
    
    let servers = service.list_servers().await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn test_mcp_service_remove_nonexistent() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let result = service.remove_server("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_service_status_no_servers() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let status = service.status(None).await;
    assert!(status.is_empty());
}

#[tokio::test]
async fn test_mcp_service_get_config() {
    let mut config = McpConfig::default();
    config.servers.push(common::sample_mcp_server("test1"));
    
    let service = McpService::new(config.clone(), None);
    let retrieved = service.get_config().await;
    
    assert_eq!(retrieved.servers.len(), 1);
    assert_eq!(retrieved.servers[0].id, "test1");
}

#[tokio::test]
async fn test_mcp_service_set_config() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let mut new_config = McpConfig::default();
    new_config.servers.push(common::sample_mcp_server("new1"));
    new_config.servers.push(common::sample_mcp_server("new2"));
    
    service.set_config(new_config).await;
    
    let servers = service.list_servers().await;
    assert_eq!(servers.len(), 2);
}

#[tokio::test]
async fn test_mcp_service_connect_nonexistent() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let result = service.connect("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_service_disconnect_nonexistent() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let result = service.disconnect("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_service_tools_list_not_connected() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("test1");
    service.add_server(server).await.unwrap();
    
    let result = service.tools_list("test1").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not connected"));
}

#[tokio::test]
async fn test_mcp_service_resources_list_not_connected() {
    let config = McpConfig::default();
    let service = McpService::new(config, None);
    
    let server = common::sample_mcp_server("test1");
    service.add_server(server).await.unwrap();
    
    let result = service.resources_list("test1").await;
    assert!(result.is_err());
}
