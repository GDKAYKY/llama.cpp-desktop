use crate::common;

use llama_desktop_lib::services::capability_registry::{CapabilityRegistry, ResolvedCall};
use llama_desktop_lib::models::McpConfig;
use llama_desktop_lib::services::mcp::service::McpService;

#[tokio::test]
async fn test_registry_new() {
    let registry = CapabilityRegistry::new();
    assert!(!registry.has_server("test").await);
}

#[tokio::test]
async fn test_registry_has_server_after_refresh() {
    let mut config = McpConfig::default();
    let mut server = common::sample_mcp_server("test1");
    server.enabled = false; // Disabled to avoid actual connection
    config.servers.push(server);
    
    let service = McpService::new(config, None);
    let registry = CapabilityRegistry::new();
    
    // Refresh with disabled server should not add it
    let _ = registry.refresh(&service).await;
    assert!(!registry.has_server("test1").await);
}

#[tokio::test]
async fn test_registry_validate_call_no_server() {
    let registry = CapabilityRegistry::new();
    
    let call = ResolvedCall {
        server_id: "nonexistent".to_string(),
        tool_name: "tool".to_string(),
        arguments: serde_json::json!({}),
    };
    
    let result = registry.validate_call(&call).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not in registry"));
}

#[tokio::test]
async fn test_registry_available_server_ids_empty() {
    let registry = CapabilityRegistry::new();
    let ids = registry.available_server_ids().await;
    assert!(ids.is_empty());
}

#[tokio::test]
async fn test_registry_summary_for_prompt_empty() {
    let registry = CapabilityRegistry::new();
    let summary = registry.summary_for_prompt(&[]).await;
    assert!(summary.contains("No tools available"));
}

#[tokio::test]
async fn test_registry_build_arguments_from_query() {
    let tool_def = serde_json::json!({
        "name": "test_tool",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            }
        }
    });
    
    let args = CapabilityRegistry::build_arguments_from_query(&tool_def, "test query");
    assert_eq!(args["query"], "test query");
}
