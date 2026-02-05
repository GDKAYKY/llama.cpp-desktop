use llama_desktop_lib::infrastructure::llama::server::LlamaServer;
use llama_desktop_lib::models::ChatRequest;
use tokio::time::timeout;
use warp::Filter;

/// Mock Server that mimics llama-server SSE behavior
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a dynamic port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

async fn run_mock_sse_server(port: u16, content_to_send: Vec<String>) -> tokio::task::JoinHandle<()> {
    let route = warp::path!("v1" / "chat" / "completions")
        .and(warp::post())
        // Validate request shape minimally if needed in the future.
        .map(move || {
            let content_chunks = content_to_send.clone();
            let stream = async_stream::stream! {
                for chunk in content_chunks {
                    let json = serde_json::json!({
                        "choices": [{
                            "delta": {
                                "content": chunk
                            }
                        }]
                    });
                    yield Ok::<_, warp::Error>(format!("data: {}\n\n", json.to_string()));
                }
                yield Ok::<_, warp::Error>("data: [DONE]\n\n".to_string());
            };
            warp::reply::with_header(
                warp::reply::Response::new(warp::hyper::Body::wrap_stream(stream)),
                "content-type",
                "text/event-stream",
            )
        });

    tokio::spawn(warp::serve(route).run(([127, 0, 0, 1], port)))
}

#[tokio::test]
async fn test_chat_parsing_logic() {
    let port = get_available_port();
    let content = vec!["Hello".to_string(), " world!".to_string()];

    // Start the mock server
    let server_handle = run_mock_sse_server(port, content.clone()).await;

    let client = reqwest::Client::new();
    let request = ChatRequest {
        model: "test-model".to_string(),
        session_id: Some("test-session".to_string()),
        messages: vec![llama_desktop_lib::models::ChatMessage {
            role: "user".to_string(),
            content: "Say hello".to_string(),
        }],
        temperature: 0.1,
        top_p: 0.95,
        top_k: 40,
        max_tokens: 16,
        stream: true,
    };

    // Exercise the SSE parsing logic in LlamaServer::stream_chat.
    let rx = LlamaServer::stream_chat(client, port, request)
        .await
        .expect("stream_chat");

    let mut collected = String::new();
    let mut rx = rx;
    let read_task = async {
        while let Some(chunk) = rx.recv().await {
            collected.push_str(&chunk);
        }
        collected
    };

    let full = timeout(std::time::Duration::from_secs(5), read_task)
        .await
        .expect("stream timeout");

    assert_eq!(full, "Hello world!");

    // Cleanup
    server_handle.abort();
}
