use crate::common;

use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use llama_desktop_lib::services::llama::service::LlamaCppService;
use llama_desktop_lib::services::mcp::service::McpService;
use llama_desktop_lib::models::McpConfig;

#[tokio::test]
async fn test_chat_session_management() {
    let dir = common::temp_dir();
    let llama_service = LlamaCppService::new(dir.path().to_path_buf());
    let mcp_service = McpService::new(McpConfig::default(), None);
    let orchestrator = ChatOrchestrator::new(llama_service, mcp_service);
    
    let session_id = "test_session";
    
    // Set history
    let messages = vec![
        common::sample_chat_message("user", "Hello"),
        common::sample_chat_message("assistant", "Hi"),
    ];
    orchestrator.set_session_history(session_id, messages).await;
    
    // Get message
    let msg = orchestrator.get_message(session_id, 0).await;
    assert!(msg.is_some());
    assert_eq!(msg.unwrap().content, "Hello");
    
    // Remove message
    let _ = orchestrator.remove_message(session_id, 0).await;
    let msg = orchestrator.get_message(session_id, 0).await;
    assert_eq!(msg.unwrap().content, "Hi");
    
    // Clear session
    orchestrator.clear_session(session_id).await;
    let msg = orchestrator.get_message(session_id, 0).await;
    assert!(msg.is_none());
}

#[tokio::test]
async fn test_chat_multiple_sessions() {
    let dir = common::temp_dir();
    let llama_service = LlamaCppService::new(dir.path().to_path_buf());
    let mcp_service = McpService::new(McpConfig::default(), None);
    let orchestrator = ChatOrchestrator::new(llama_service, mcp_service);
    
    let session1 = "session1";
    let session2 = "session2";
    
    orchestrator.set_session_history(session1, vec![
        common::sample_chat_message("user", "Session 1 message"),
    ]).await;
    
    orchestrator.set_session_history(session2, vec![
        common::sample_chat_message("user", "Session 2 message"),
    ]).await;
    
    let msg1 = orchestrator.get_message(session1, 0).await.unwrap();
    let msg2 = orchestrator.get_message(session2, 0).await.unwrap();
    
    assert_eq!(msg1.content, "Session 1 message");
    assert_eq!(msg2.content, "Session 2 message");
}
