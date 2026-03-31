use crate::common;

use llama_desktop_lib::models::*;
use serde_json;

#[test]
fn test_model_id_display() {
    let id = ModelId("test:model:v1".to_string());
    assert_eq!(format!("{}", id), "test:model:v1");
}

#[test]
fn test_llama_config_serialization() {
    let config = common::sample_llama_config();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: LlamaCppConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.llama_cpp_path, deserialized.llama_cpp_path);
    assert_eq!(config.model_path, deserialized.model_path);
    assert_eq!(config.port, deserialized.port);
    assert_eq!(config.ctx_size, deserialized.ctx_size);
}

#[test]
fn test_server_metrics_serialization() {
    let metrics = ServerMetrics {
        cpu_usage: 45.5,
        mem_usage: 1024,
        gpu_usage: Some(30.0),
        vram_usage: Some(50.0),
    };
    
    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: ServerMetrics = serde_json::from_str(&json).unwrap();
    
    assert_eq!(metrics.cpu_usage, deserialized.cpu_usage);
    assert_eq!(metrics.mem_usage, deserialized.mem_usage);
    assert_eq!(metrics.gpu_usage, deserialized.gpu_usage);
}

#[test]
fn test_mcp_transport_serialization() {
    let stdio = McpTransport::Stdio;
    let http = McpTransport::HttpSse;
    
    assert_eq!(serde_json::to_string(&stdio).unwrap(), "\"stdio\"");
    assert_eq!(serde_json::to_string(&http).unwrap(), "\"http_sse\"");
}

#[test]
fn test_mcp_server_config_serialization() {
    let server = common::sample_mcp_server("test1");
    let json = serde_json::to_string(&server).unwrap();
    let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(server.id, deserialized.id);
    assert_eq!(server.name, deserialized.name);
    assert_eq!(server.enabled, deserialized.enabled);
}

#[test]
fn test_mcp_config_default() {
    let config = McpConfig::default();
    assert!(config.servers.is_empty());
}

#[test]
fn test_chat_message_serialization() {
    let msg = common::sample_chat_message("user", "Hello");
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(msg.role, deserialized.role);
    assert_eq!(msg.content, deserialized.content);
}

#[test]
fn test_model_manifest_serialization() {
    let manifest = common::create_test_model_manifest();
    let json = serde_json::to_string(&manifest).unwrap();
    let deserialized: ModelManifest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(manifest.schema_version, deserialized.schema_version);
    assert_eq!(manifest.config.digest, deserialized.config.digest);
    assert_eq!(manifest.layers.len(), deserialized.layers.len());
}

#[test]
fn test_model_info_full_identifier() {
    let info = common::create_test_model_info();
    assert_eq!(info.full_identifier, "test:model:v1");
    assert_eq!(info.provider, "test");
    assert_eq!(info.name, "model");
    assert_eq!(info.version, "v1");
}

#[test]
fn test_model_library_serialization() {
    let library = ModelLibrary {
        models: vec![common::create_test_model_info()],
    };
    
    let json = serde_json::to_string(&library).unwrap();
    let deserialized: ModelLibrary = serde_json::from_str(&json).unwrap();
    
    assert_eq!(library.models.len(), deserialized.models.len());
    assert_eq!(library.models[0].full_identifier, deserialized.models[0].full_identifier);
}

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert!(config.models_directory.is_none());
    assert!(config.llama_directory.is_none());
}

#[test]
fn test_app_config_serialization() {
    let config = AppConfig {
        models_directory: Some("/models".to_string()),
        llama_directory: Some("/bin/llama".to_string()),
        ..Default::default()
    };
    
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.models_directory, deserialized.models_directory);
    assert_eq!(config.llama_directory, deserialized.llama_directory);
}
