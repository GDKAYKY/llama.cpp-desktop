use llama_desktop_lib::models::*;
use tempfile::TempDir;
use tokio::process::{Child, Command};

pub fn sample_llama_config() -> LlamaCppConfig {
    LlamaCppConfig {
        llama_cpp_path: "llama-server".to_string(),
        model_path: "/models/test.gguf".to_string(),
        port: 8080,
        ctx_size: 2048,
        parallel: 1,
        n_gpu_layers: 0,
        chat_template: None,
        chat_template_file: None,
    }
}

pub fn sample_mcp_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: format!("Test Server {}", id),
        enabled: true,
        transport: McpTransport::Stdio,
        command: Some("node".to_string()),
        args: Some(vec!["server.js".to_string()]),
        cwd: None,
        env: None,
        url: None,
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    }
}

pub fn sample_chat_message(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }
}

pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

pub fn spawn_sleep_process() -> std::io::Result<Child> {
    if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg("timeout")
            .arg("/T")
            .arg("1000")
            .arg("/NOBREAK")
            .spawn()
    } else {
        Command::new("sleep").arg("1000").spawn()
    }
}

pub fn create_test_model_manifest() -> ModelManifest {
    ModelManifest {
        schema_version: 2,
        media_type: "application/vnd.ollama.manifest.v1+json".to_string(),
        config: ManifestConfig {
            media_type: "application/vnd.ollama.image.config".to_string(),
            digest: "sha256:abc123".to_string(),
            size: 100,
        },
        layers: vec![ManifestLayer {
            media_type: "application/vnd.ollama.image.model".to_string(),
            digest: "sha256:def456".to_string(),
            size: 1000,
        }],
    }
}

pub fn create_test_model_info() -> ModelInfo {
    ModelInfo {
        provider: "test".to_string(),
        library: "library".to_string(),
        name: "model".to_string(),
        version: "v1".to_string(),
        manifest_data: create_test_model_manifest(),
        tokenizer_metadata: None,
        model_file_path: Some("/models/test.gguf".to_string()),
        manifest_path: Some("manifests/test/library/model/v1/manifest.json".to_string()),
        full_identifier: "test:model:v1".to_string(),
    }
}
