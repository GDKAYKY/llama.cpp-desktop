use crate::common;

fn create_test_orchestrator() -> ChatOrchestrator {
    let temp_dir = common::temp_dir();
    let llama_service = LlamaCppService::new(temp_dir.path().to_path_buf());
    let mcp_service = McpService::new(McpConfig::default(), None);
    ChatOrchestrator::new(llama_service, mcp_service)
}

use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use llama_desktop_lib::services::llama::service::LlamaCppService;
use llama_desktop_lib::services::mcp::service::McpService;
use llama_desktop_lib::models::McpConfig;


#[tokio::test]
async fn test_orchestrator_new() {
    let orchestrator = create_test_orchestrator();
    orchestrator.clear_session("test").await;
    // Should not panic
}

#[tokio::test]
async fn test_orchestrator_set_and_get_message() {
    let orchestrator = create_test_orchestrator();
    let session_id = "test_session";
    
    let messages = vec![
        common::sample_chat_message("user", "Hello"),
        common::sample_chat_message("assistant", "Hi there"),
    ];
    
    orchestrator.set_session_history(session_id, messages.clone()).await;
    
    let msg = orchestrator.get_message(session_id, 0).await;
    assert!(msg.is_some());
    assert_eq!(msg.unwrap().content, "Hello");
    
    let msg2 = orchestrator.get_message(session_id, 1).await;
    assert!(msg2.is_some());
    assert_eq!(msg2.unwrap().content, "Hi there");
}

#[tokio::test]
async fn test_orchestrator_get_message_out_of_bounds() {
    let orchestrator = create_test_orchestrator();
    let session_id = "test_session";
    
    let messages = vec![common::sample_chat_message("user", "Hello")];
    orchestrator.set_session_history(session_id, messages).await;
    
    let msg = orchestrator.get_message(session_id, 10).await;
    assert!(msg.is_none());
}

#[tokio::test]
async fn test_orchestrator_remove_message() {
    let orchestrator = create_test_orchestrator();
    let session_id = "test_session";
    
    let messages = vec![
        common::sample_chat_message("user", "First"),
        common::sample_chat_message("assistant", "Second"),
        common::sample_chat_message("user", "Third"),
    ];
    
    orchestrator.set_session_history(session_id, messages).await;
    let _ = orchestrator.remove_message(session_id, 1).await;
    
    let msg = orchestrator.get_message(session_id, 1).await;
    assert!(msg.is_some());
    assert_eq!(msg.unwrap().content, "Third");
}

#[tokio::test]
async fn test_orchestrator_clear_session() {
    let orchestrator = create_test_orchestrator();
    let session_id = "test_session";
    
    let messages = vec![common::sample_chat_message("user", "Hello")];
    orchestrator.set_session_history(session_id, messages).await;
    
    orchestrator.clear_session(session_id).await;
    
    let msg = orchestrator.get_message(session_id, 0).await;
    assert!(msg.is_none());
}

#[tokio::test]
async fn test_orchestrator_prepare_regenerate_history() {
    let messages = vec![
        common::sample_chat_message("user", "Q1"),
        common::sample_chat_message("assistant", "A1"),
        common::sample_chat_message("user", "Q2"),
        common::sample_chat_message("assistant", "A2"),
    ];
    
    // Changed index from 2 (user message) to 3 (assistant message "A2")
    let result = ChatOrchestrator::prepare_regenerate_history(&messages, 3).unwrap();
    
    assert_eq!(result.len(), 3);
    assert_eq!(result[2].content, "Q2");
}

#[tokio::test]
async fn test_orchestrator_refresh_capabilities() {
    let orchestrator = create_test_orchestrator();
    let result = orchestrator.refresh_capabilities().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_orchestrator_complete_chat_once_no_model() {
    let orchestrator = create_test_orchestrator();
    
    let messages = vec![common::sample_chat_message("user", "Hello")];
    let result = orchestrator.complete_chat_once(messages, 0.7, 1.0, 40, 512, None, None, None).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No model running"));
}