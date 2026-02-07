use llama_desktop_lib::services::mcp::client::{HttpClient, McpClient};
use serde_json::json;
use std::collections::HashMap;
mod mcp_stdio;

use tokio::time::{sleep, Duration};
use warp::Filter;

#[tokio::test]
async fn stdio_client_request_success() {
    let (client, _handle) = mcp_stdio::spawn_in_memory_stdio_client();
    let result = client.request("ping", None).await.expect("request");
    assert_eq!(result["echo"], "ping");
    client.shutdown().await;
}

#[tokio::test]
async fn stdio_client_request_error() {
    let (client, _handle) = mcp_stdio::spawn_in_memory_stdio_client();
    let err = client.request("error", None).await.expect_err("expected error");
    assert!(err.contains("MCP error"));
    client.shutdown().await;
}

#[tokio::test]
async fn stdio_client_skips_mismatched_ids() {
    let (client, _handle) = mcp_stdio::spawn_in_memory_stdio_client();
    let result = client.request("wrong_id", None).await.expect("request");
    assert_eq!(result["echo"], "right");
    client.shutdown().await;
}

#[tokio::test]
async fn http_client_request_success() {
    let route = warp::post().and(warp::body::json()).map(|body: serde_json::Value| {
        let id = body.get("id").cloned().unwrap_or(serde_json::Value::Null);
        warp::reply::json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "ok": true }
        }))
    });
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let mut headers = HashMap::new();
    headers.insert("X-Test".to_string(), "1".to_string());
    let client = McpClient::connect_http_sse(&format!("http://{}", addr), Some(headers))
        .await
        .expect("client");
    let result = client.request("anything", None).await.expect("request");
    assert_eq!(result["ok"], true);
    client.shutdown().await;
}

#[tokio::test]
async fn http_client_request_error() {
    let route = warp::post().and(warp::body::json()).map(|body: serde_json::Value| {
        let id = body.get("id").cloned().unwrap_or(serde_json::Value::Null);
        warp::reply::json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -10, "message": "bad" }
        }))
    });
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let client = HttpClient::new(format!("http://{}", addr), None).expect("client");
    let err = client.request("anything", None).await.expect_err("error");
    assert!(err.contains("MCP error"));
}

#[tokio::test]
async fn stdio_shutdown_kills_process() {
    let (client, _handle) = mcp_stdio::spawn_in_memory_stdio_client();
    client.shutdown().await;
    sleep(Duration::from_millis(10)).await;
}

#[tokio::test]
async fn stdio_client_reports_closed_stdout() {
    let (client_to_server, _server_from_client) = tokio::io::duplex(1024);
    let (server_to_client, client_from_server) = tokio::io::duplex(1024);
    drop(server_to_client);
    let client = McpClient::connect_stdio_with_io(client_to_server, client_from_server);
    let err = client.request("ping", None).await.expect_err("error");
    assert!(err.contains("closed stdout"));
    client.shutdown().await;
}
