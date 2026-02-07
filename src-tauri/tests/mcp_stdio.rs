use llama_desktop_lib::services::mcp::client::McpClient;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::task::JoinHandle;

pub fn spawn_in_memory_stdio_client() -> (McpClient, JoinHandle<()>) {
    let (client_to_server, server_from_client) = tokio::io::duplex(1024);
    let (server_to_client, client_from_server) = tokio::io::duplex(1024);

    let client = McpClient::connect_stdio_with_io(client_to_server, client_from_server);
    let handle = tokio::spawn(async move {
        let mut reader = BufReader::new(server_from_client);
        let mut writer = server_to_client;
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = match reader.read_line(&mut line).await {
                Ok(bytes) => bytes,
                Err(_) => break,
            };
            if bytes == 0 {
                break;
            }

            let parsed: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let id = parsed.get("id").cloned().unwrap_or(Value::Null);
            let method = parsed
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let responses = match method {
                "error" => vec![json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -1, "message": "boom" }
                })],
                "wrong_id" => {
                    let wrong_id = id
                        .as_u64()
                        .map(|v| json!(v + 1))
                        .unwrap_or_else(|| json!(1));
                    vec![
                        json!({
                            "jsonrpc": "2.0",
                            "id": wrong_id,
                            "result": { "echo": "wrong" }
                        }),
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "echo": "right" }
                        }),
                    ]
                }
                "tools/list" => vec![json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            { "name": "echo" },
                            { "name": "hidden" }
                        ]
                    }
                })],
                "resources/list" => vec![json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "resources": [
                            { "uri": "file://one" },
                            { "uri": "file://two" }
                        ]
                    }
                })],
                _ => vec![json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "echo": method }
                })],
            };

            let _ = writer.write_all(b"junk\n").await;
            for response in responses {
                let line = serde_json::to_string(&response).unwrap_or_default();
                let _ = writer.write_all(line.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
            }
            let _ = writer.flush().await;
        }
    });

    (client, handle)
}
