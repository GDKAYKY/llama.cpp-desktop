use llama_desktop_lib::models::{McpConfig, McpServerConfig, McpTransport};
use llama_desktop_lib::services::mcp::service::McpService;
use serde_json::json;
use tokio::time::{timeout, Duration};
use warp::Filter;

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
                        {"name": "allowed"},
                        {"name": "blocked"}
                    ]
                }),
                "tools/call" => json!({
                    "ok": true,
                    "echo": req.get("params").cloned().unwrap_or(json!({}))
                }),
                "resources/list" => json!({
                    "resources": [
                        {"uri": "res://ok"},
                        {"uri": "res://nope"}
                    ]
                }),
                "resources/read" => json!({
                    "contents": "hello"
                }),
                _ => json!({
                    "unknown": true
                }),
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

#[tokio::test]
async fn test_mcp_http_integration_and_allowlists() {
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
            tool_allowlist: Some(vec!["allowed".to_string()]),
            resource_allowlist: Some(vec!["res://ok".to_string()]),
        }],
    };

    let service = McpService::new(cfg);

    timeout(Duration::from_secs(2), service.connect("local"))
        .await
        .expect("connect timeout")
        .expect("connect");

    let tools = service.tools_list("local").await.expect("tools_list");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "allowed");

    let res = service
        .resources_list("local")
        .await
        .expect("resources_list");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["uri"], "res://ok");

    let call = service
        .tools_call("local", "allowed", json!({"x": 1}))
        .await
        .expect("tools_call");
    assert_eq!(call["ok"], true);
    assert_eq!(call["echo"]["name"], "allowed");
    assert_eq!(call["echo"]["arguments"]["x"], 1);

    let err = service
        .tools_call("local", "blocked", json!({}))
        .await
        .expect_err("should reject blocked tool");
    assert!(err.contains("Tool not allowed"));

    let read = service
        .resources_read("local", "res://ok")
        .await
        .expect("resources_read");
    assert_eq!(read["contents"], "hello");

    let err = service
        .resources_read("local", "res://nope")
        .await
        .expect_err("should reject blocked resource");
    assert!(err.contains("Resource not allowed"));

    service.disconnect("local").await.expect("disconnect");

    server_handle.abort();
}
