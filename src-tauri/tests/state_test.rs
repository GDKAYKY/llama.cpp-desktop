use llama_desktop_lib::models::McpConfig;
use llama_desktop_lib::state::AppState;
use tempfile::tempdir;

#[tokio::test]
async fn app_state_new_initializes_services() {
    let dir = tempdir().expect("tempdir");
    let state = AppState::new(dir.path().to_path_buf(), McpConfig::default());
    let config = state.mcp_service.get_config().await;
    assert!(config.servers.is_empty());
}
