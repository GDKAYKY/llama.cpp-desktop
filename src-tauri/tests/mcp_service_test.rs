use llama_desktop_lib::models::{
    McpConfig, McpServerConfig, McpTransport, ResourceDefinition, ToolDefinition,
};
use llama_desktop_lib::services::mcp::service::McpService;
use serde_json::json;
use warp::Filter;

mod mcp_stdio;

fn sample_server(id: &str, url: String) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: "Test".to_string(),
        enabled: true,
        transport: McpTransport::HttpSse,
        command: None,
        args: None,
        cwd: None,
        env: None,
        url: Some(url),
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    }
}

fn start_rpc_server() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let route = warp::post().and(warp::body::json()).map(|body: serde_json::Value| {
        let id = body.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = body
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let result = match method {
            "tools/list" => json!({
                "tools": [
                    { "name": "alpha" },
                    { "name": "beta" }
                ]
            }),
            "resources/list" => json!({
                "resources": [
                    { "uri": "file://one" },
                    { "uri": "file://two" }
                ]
            }),
            "tools/call" => json!({ "ok": true }),
            "resources/read" => json!({ "content": "data" }),
            _ => json!({ "ok": false }),
        };
        warp::reply::json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    });

    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    let handle = tokio::spawn(server);
    (addr, handle)
}

fn stdio_service(config: McpConfig) -> McpService {
    McpService::new_with_stdio_connector(config, |_command, _args, _cwd, _env| async move {
        let (client, _handle) = mcp_stdio::spawn_in_memory_stdio_client();
        Ok(client)
    })
}

#[tokio::test]
async fn add_update_remove_server_flow() {
    let service = McpService::new(McpConfig::default());
    let server = McpServerConfig {
        id: "one".to_string(),
        name: "Server".to_string(),
        enabled: true,
        transport: McpTransport::HttpSse,
        command: None,
        args: None,
        cwd: None,
        env: None,
        url: Some("http://localhost".to_string()),
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    };

    service.add_server(server.clone()).await.expect("add");
    let err = service.add_server(server.clone()).await.expect_err("dup");
    assert!(err.contains("same id"));

    let mut updated = server.clone();
    updated.name = "Updated".to_string();
    service.update_server(updated).await.expect("update");

    service.remove_server("one").await.expect("remove");
}

#[tokio::test]
async fn remove_server_disconnects_active_connection() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    config.servers.push(sample_server("one", format!("http://{}", addr)));
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    service.remove_server("one").await.expect("remove");
    let status = service.status(Some("one".to_string())).await;
    assert_eq!(status.len(), 1);
    assert!(!status[0].connected);
}

#[tokio::test]
async fn set_and_get_config_updates_state() {
    let service = McpService::new(McpConfig::default());
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    service.set_config(config.clone()).await;
    let loaded = service.get_config().await;
    assert_eq!(loaded.servers.len(), 1);
}

#[tokio::test]
async fn list_servers_returns_configured_servers() {
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    let service = McpService::new(config);
    let servers = service.list_servers().await;
    assert_eq!(servers.len(), 1);
}

#[tokio::test]
async fn connect_and_list_tools_with_allowlist() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    let mut server = sample_server("one", format!("http://{}", addr));
    server.tool_allowlist = Some(vec!["beta".to_string()]);
    config.servers.push(server);
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    let tools = service.tools_list("one").await.expect("tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "beta");
}

#[tokio::test]
async fn tools_list_errors_when_not_connected() {
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    let service = McpService::new(config);
    let err = service.tools_list("one").await.expect_err("not connected");
    assert!(err.contains("Server not connected"));
}

#[tokio::test]
async fn tools_list_errors_when_server_missing() {
    let service = McpService::new(McpConfig::default());
    let err = service.tools_list("missing").await.expect_err("missing");
    assert!(err.contains("Server not found"));
}

#[tokio::test]
async fn connect_and_list_resources_with_allowlist() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    let mut server = sample_server("one", format!("http://{}", addr));
    server.resource_allowlist = Some(vec!["file://two".to_string()]);
    config.servers.push(server);
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    let resources = service.resources_list("one").await.expect("resources");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0]["uri"], "file://two");
}

#[tokio::test]
async fn resources_list_errors_when_not_connected() {
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    let service = McpService::new(config);
    let err = service
        .resources_list("one")
        .await
        .expect_err("not connected");
    assert!(err.contains("Server not connected"));
}

#[tokio::test]
async fn resources_read_errors_when_not_connected() {
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    let service = McpService::new(config);
    let err = service
        .resources_read("one", "file://one")
        .await
        .expect_err("not connected");
    assert!(err.contains("Server not connected"));
}

#[tokio::test]
async fn tools_call_and_resource_read_respect_allowlist() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    let mut server = sample_server("one", format!("http://{}", addr));
    server.tool_allowlist = Some(vec!["alpha".to_string()]);
    server.resource_allowlist = Some(vec!["file://one".to_string()]);
    config.servers.push(server);
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    let ok = service
        .tools_call("one", "alpha", json!({}))
        .await
        .expect("call");
    assert_eq!(ok["ok"], true);

    let err = service
        .tools_call("one", "beta", json!({}))
        .await
        .expect_err("blocked");
    assert!(err.contains("Tool not allowed"));

    let read = service.resources_read("one", "file://one").await.expect("read");
    assert_eq!(read["content"], "data");

    let err = service
        .resources_read("one", "file://two")
        .await
        .expect_err("blocked");
    assert!(err.contains("Resource not allowed"));
}

#[tokio::test]
async fn tools_call_errors_when_not_connected() {
    let config = McpConfig {
        servers: vec![McpServerConfig {
            id: "one".to_string(),
            name: "Server".to_string(),
            enabled: true,
            transport: McpTransport::HttpSse,
            command: None,
            args: None,
            cwd: None,
            env: None,
            url: Some("http://localhost".to_string()),
            headers: None,
            tool_allowlist: None,
            resource_allowlist: None,
        }],
    };
    let service = McpService::new(config);
    let err = service
        .tools_call("one", "alpha", json!({}))
        .await
        .expect_err("not connected");
    assert!(err.contains("Server not connected"));
}

#[tokio::test]
async fn tools_and_resources_allow_when_no_allowlist() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    let server = sample_server("one", format!("http://{}", addr));
    config.servers.push(server);
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    let ok = service
        .tools_call("one", "alpha", json!({}))
        .await
        .expect("call");
    assert_eq!(ok["ok"], true);

    let read = service
        .resources_read("one", "file://one")
        .await
        .expect("read");
    assert_eq!(read["content"], "data");
}

#[tokio::test]
async fn status_reports_connected_and_disconnected() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    config.servers.push(sample_server("one", format!("http://{}", addr)));
    config.servers.push(sample_server("two", format!("http://{}", addr)));
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    let status = service.status(None).await;
    let one = status.iter().find(|s| s.id == "one").expect("one");
    let two = status.iter().find(|s| s.id == "two").expect("two");
    assert!(one.connected);
    assert!(!two.connected);
}

#[tokio::test]
async fn disconnect_removes_connection() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    config.servers.push(sample_server("one", format!("http://{}", addr)));
    let service = McpService::new(config);

    service.connect("one").await.expect("connect");
    service.disconnect("one").await.expect("disconnect");
    let status = service.status(Some("one".to_string())).await;
    assert_eq!(status.len(), 1);
    assert!(!status[0].connected);
}

#[tokio::test]
async fn status_filters_by_id() {
    let (addr, _server) = start_rpc_server();
    let mut config = McpConfig::default();
    config.servers.push(sample_server("one", format!("http://{}", addr)));
    config.servers.push(sample_server("two", format!("http://{}", addr)));
    let service = McpService::new(config);
    let status = service.status(Some("two".to_string())).await;
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].id, "two");
}

#[tokio::test]
async fn connect_rejects_missing_config() {
    let service = McpService::new(McpConfig::default());
    let err = service.connect("missing").await.expect_err("no server");
    assert!(err.contains("Server not found"));
}

#[tokio::test]
async fn connect_rejects_disabled_server() {
    let mut config = McpConfig::default();
    let mut server = sample_server("one", "http://localhost".to_string());
    server.enabled = false;
    config.servers.push(server);
    let service = McpService::new(config);
    let err = service.connect("one").await.expect_err("disabled");
    assert!(err.contains("disabled"));
}

#[tokio::test]
async fn connect_rejects_missing_stdio_command() {
    let mut config = McpConfig::default();
    config.servers.push(McpServerConfig {
        id: "stdio".to_string(),
        name: "stdio".to_string(),
        enabled: true,
        transport: McpTransport::Stdio,
        command: None,
        args: None,
        cwd: None,
        env: None,
        url: None,
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    });
    let service = McpService::new(config);
    let err = service.connect("stdio").await.expect_err("missing command");
    assert!(err.contains("Missing command"));
}

#[tokio::test]
async fn connect_rejects_missing_http_url() {
    let mut config = McpConfig::default();
    config.servers.push(McpServerConfig {
        id: "http".to_string(),
        name: "http".to_string(),
        enabled: true,
        transport: McpTransport::HttpSse,
        command: None,
        args: None,
        cwd: None,
        env: None,
        url: None,
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    });
    let service = McpService::new(config);
    let err = service.connect("http").await.expect_err("missing url");
    assert!(err.contains("Missing url"));
}

#[tokio::test]
async fn connect_stdio_server_and_list_tools() {
    let mut config = McpConfig::default();
    config.servers.push(McpServerConfig {
        id: "stdio".to_string(),
        name: "stdio".to_string(),
        enabled: true,
        transport: McpTransport::Stdio,
        command: Some("in-memory".to_string()),
        args: Some(Vec::new()),
        cwd: None,
        env: None,
        url: None,
        headers: None,
        tool_allowlist: Some(vec!["echo".to_string()]),
        resource_allowlist: None,
    });
    let service = stdio_service(config);
    service.connect("stdio").await.expect("connect");
    let tools: Vec<ToolDefinition> = service.tools_list("stdio").await.expect("tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo");
    let resources: Vec<ResourceDefinition> =
        service.resources_list("stdio").await.expect("resources");
    assert_eq!(resources.len(), 2);
}

#[tokio::test]
async fn connect_stdio_server_without_args() {
    let mut config = McpConfig::default();
    config.servers.push(McpServerConfig {
        id: "stdio".to_string(),
        name: "stdio".to_string(),
        enabled: true,
        transport: McpTransport::Stdio,
        command: Some("in-memory".to_string()),
        args: None,
        cwd: None,
        env: None,
        url: None,
        headers: None,
        tool_allowlist: None,
        resource_allowlist: None,
    });
    let service = stdio_service(config);
    service.connect("stdio").await.expect("connect");
    let tools = service.tools_list("stdio").await.expect("tools");
    assert!(!tools.is_empty());
}
