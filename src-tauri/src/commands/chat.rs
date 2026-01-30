use crate::state::AppState;
use futures::StreamExt; // Required for stream.next()
use serde_json::json;
use std::io::BufRead;
use tauri::{ipc::Channel, State};

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    _session_id: String,
    message: String,
    temperature: f32,
    max_tokens: i32,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    // 1. Get config directly (port)
    let config = state
        .llama_service
        .get_config()
        .await
        .ok_or("Server not running")?;
    let port = config.port;

    // 2. Prepare payload
    let payload = json!({
        "model": "llama",
        "messages": [
            { "role": "user", "content": message }
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream": true
    });

    // 3. Send Request
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://localhost:{}/v1/chat/completions", port))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Server error: {}", res.status()));
    }

    // 4. Stream Response
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        let s = String::from_utf8_lossy(&chunk);

        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    let _ = on_event.send(json!({ "status": "done" }));
                    break;
                }
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        let _ = on_event.send(json!({ "chunk": content }));
                    }
                }
            }
        }
    }

    Ok(())
}
