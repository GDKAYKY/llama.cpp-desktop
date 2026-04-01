use llama_desktop_lib::models::{ChatMessage, McpConfig, McpServerConfig, McpTransport};
use llama_desktop_lib::services::llama::ActorMessage;
use llama_desktop_lib::services::llama::LlamaCppService;
use llama_desktop_lib::services::mcp::McpService;
use llama_desktop_lib::services::orchestrator::ChatOrchestrator;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use warp::Filter;

fn sample_config() -> llama_desktop_lib::models::LlamaCppConfig {
    llama_desktop_lib::models::LlamaCppConfig {
        llama_cpp_path: "llama".to_string(),
        model_path: "model".to_string(),
        port: 8080,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
        chat_template: None,
        chat_template_file: None,
    }
}

fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a dynamic port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

async fn run_mock_mcp_server(port: u16) -> tokio::task::JoinHandle<()> {
    let route = warp::path!("mcp")
        .and(warp::post())
        .and(warp::body::json())
        .map(|req: serde_json::Value| {
            let id = req.get("id").cloned().unwrap_or_else(|| json!(1));
            let method = req
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let result = match method {
                "tools/list" => json!({
                    "tools": [
                        { "name": "allowed", "description": "Allowed tool", "inputSchema": { "type": "object", "properties": { "x": { "type": "number" } } } }
                    ]
                }),
                "tools/call" => json!({
                    "ok": true,
                    "echo": req.get("params").cloned().unwrap_or(json!({}))
                }),
                "resources/list" => json!({ "resources": [] }),
                _ => json!({ "unknown": true }),
            };

            let resp = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            });
            warp::reply::json(&resp)
        });

    tokio::spawn(warp::serve(route).run(([127, 0, 0, 1], port)))
}

async fn wait_for_port(port: u16, timeout_ms: u64) {
    let start = std::time::Instant::now();
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        if start.elapsed() > Duration::from_millis(timeout_ms) {
            panic!("mock MCP server did not start listening on port {}", port);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn test_channel() -> (Channel<serde_json::Value>, mpsc::Receiver<serde_json::Value>) {
    let (tx, rx) = mpsc::channel::<serde_json::Value>(8);
    let channel = Channel::new(move |value: serde_json::Value| -> tauri::Result<()> {
        let _ = tx.try_send(value);
        Ok(())
    });

    (channel, rx)
}

fn mock_service(chunks: Vec<&'static str>) -> LlamaCppService {
    let (tx, mut rx) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let (out_tx, out_rx) = mpsc::channel(8);
                    for chunk in &chunks {
                        let _ = out_tx.send(chunk.to_string()).await;
                    }
                    drop(out_tx);
                    let _ = respond_to.send(Ok(out_rx));
                }
                _ => {}
            }
        }
    });
    LlamaCppService::from_sender(tx)
}

fn mock_service_with_tool_calls() -> LlamaCppService {
    let (tx, mut rx) = mpsc::channel(8);
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::CompleteChat { respond_to, .. } => {
                    let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
                    let response = if count == 0 {
                        json!({
                            "choices": [
                                { "message": { "content": "", "tool_calls": [
                                    { "id": "call-1", "type": "function",
                                      "function": { "name": "mcp__local__allowed", "arguments": "{\"x\": 1}" } }
                                ] } }
                            ]
                        })
                    } else {
                        json!({
                            "choices": [
                                { "message": { "content": "ready" } }
                            ]
                        })
                    };
                    let _ = respond_to.send(Ok(response));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let (out_tx, out_rx) = mpsc::channel(8);
                    let _ = out_tx.send("final".to_string()).await;
                    drop(out_tx);
                    let _ = respond_to.send(Ok(out_rx));
                }
                _ => {}
            }
        }
    });

    LlamaCppService::from_sender(tx)
}

#[tokio::test]
async fn prepare_regenerate_history_validates_role() {
    let history = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "hello".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    ];

    let err = ChatOrchestrator::prepare_regenerate_history(&history, 0)
        .expect_err("not assistant");
    assert!(err.contains("Target message is not an assistant"));

    let ok = ChatOrchestrator::prepare_regenerate_history(&history, 1).expect("ok");
    assert_eq!(ok.len(), 1);
}

#[tokio::test]
async fn prepare_regenerate_history_errors_when_missing() {
    let history = vec![ChatMessage {
        role: "assistant".to_string(),
        content: "hi".to_string(),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    let err = ChatOrchestrator::prepare_regenerate_history(&history, 2).expect_err("missing");
    assert!(err.contains("Message not found"));
}

#[tokio::test]
async fn process_appends_messages_and_updates_history() {
    let service = mock_service(vec!["Hello", " world"]);
    let mcp_service = McpService::new(McpConfig::default());
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, mut rx) = test_channel();

    orchestrator
        .process("session-1", "Hi".to_string(), 0.5, 10, channel)
        .await
        .expect("process");

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    assert!(events.iter().any(|e| e.get("chunk").is_some()));

    let message = orchestrator.get_message("session-1", 1).await;
    assert_eq!(message.map(|m| m.content), Some("Hello world".to_string()));
}

#[tokio::test]
async fn process_propagates_send_errors() {
    let (tx, mut rx) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let _ = respond_to.send(Err("boom".to_string()));
                }
                _ => {}
            }
        }
    });
    let service = LlamaCppService::from_sender(tx);
    let mcp_service = McpService::new(McpConfig::default());
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, _rx) = test_channel();

    let err = orchestrator
        .process("session-1", "Hi".to_string(), 0.5, 10, channel)
        .await
        .expect_err("error");
    assert!(err.contains("boom"));
}

#[tokio::test]
async fn regenerate_at_replaces_message() {
    let service = mock_service(vec!["New", " answer"]);
    let mcp_service = McpService::new(McpConfig::default());
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, _rx) = test_channel();
    orchestrator
        .set_session_history(
            "session-1",
            vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Ask".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Old".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
            ],
        )
        .await;

    orchestrator
        .regenerate_at("session-1", 1, 0.5, 10, channel)
        .await
        .expect("regenerate");

    let message = orchestrator.get_message("session-1", 1).await;
    assert_eq!(message.map(|m| m.content), Some("New answer".to_string()));
}

#[tokio::test]
async fn regenerate_at_errors_when_session_missing() {
    let service = mock_service(vec!["ok"]);
    let mcp_service = McpService::new(McpConfig::default());
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, _rx) = test_channel();
    let err = orchestrator
        .regenerate_at("missing", 0, 0.5, 10, channel)
        .await
        .expect_err("error");
    assert!(err.contains("Session not found"));
}

#[tokio::test]
async fn regenerate_at_errors_when_message_removed() {
    let (tx, mut rx) = mpsc::channel(8);
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let mut gate_rx = Some(gate_rx);
        while let Some(msg) = rx.recv().await {
            match msg {
                ActorMessage::GetConfig { respond_to } => {
                    let _ = respond_to.send(Some(sample_config()));
                }
                ActorMessage::SendChat { respond_to, .. } => {
                    let (out_tx, out_rx) = mpsc::channel(8);
                    let _ = respond_to.send(Ok(out_rx));
                    if let Some(rx) = gate_rx.take() {
                        let _ = rx.await;
                    }
                    let _ = out_tx.send("done".to_string()).await;
                }
                _ => {}
            }
        }
    });
    let service = LlamaCppService::from_sender(tx);
    let mcp_service = McpService::new(McpConfig::default());
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, _rx) = test_channel();
    orchestrator
        .set_session_history(
            "session-1",
            vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Ask".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Old".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
            ],
        )
        .await;

    let handle = tokio::spawn({
        let orchestrator = orchestrator.clone();
        async move { orchestrator.regenerate_at("session-1", 1, 0.5, 10, channel).await }
    });

    orchestrator
        .remove_message("session-1", 1)
        .await
        .expect("remove");
    let _ = gate_tx.send(());

    let err = handle.await.expect("handle").expect_err("error");
    assert!(err.contains("Message removed"));
}

#[tokio::test]
async fn process_executes_tool_loop_and_streams() {
    let port = get_available_port();
    let server_handle = run_mock_mcp_server(port).await;
    wait_for_port(port, 1000).await;

    let cfg = McpConfig {
        servers: vec![McpServerConfig {
            id: "local".to_string(),
            name: "Local MCP".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some(format!("http://127.0.0.1:{}/mcp", port)),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };

    let service = mock_service_with_tool_calls();
    let mcp_service = McpService::new(cfg);
    let orchestrator = ChatOrchestrator::new(service, mcp_service);
    let (channel, mut rx) = test_channel();

    timeout(Duration::from_secs(2), orchestrator.refresh_capabilities())
        .await
        .expect("refresh timeout")
        .expect("refresh");

    orchestrator
        .process("session-1", "Use tool".to_string(), 0.2, 64, channel)
        .await
        .expect("process");

    let mut chunks = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Some(chunk) = event.get("chunk").and_then(|v| v.as_str()) {
            chunks.push(chunk.to_string());
        }
    }
    assert!(!chunks.is_empty());

    let tool_msg = orchestrator.get_message("session-1", 2).await;
    assert_eq!(tool_msg.map(|m| m.role), Some("tool".to_string()));

    let final_msg = orchestrator.get_message("session-1", 3).await;
    assert_eq!(final_msg.map(|m| m.content), Some("final".to_string()));

    server_handle.abort();
}
