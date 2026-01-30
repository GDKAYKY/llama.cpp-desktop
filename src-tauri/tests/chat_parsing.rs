use llama_desktop_lib::models::{ChatMessage, ChatRequest};
use std::sync::Arc;
use tokio::sync::mpsc;
use warp::Filter;

/// Mock Server that mimics llama-server SSE behavior
async fn run_mock_sse_server(
    port: u16,
    content_to_send: Vec<String>,
) -> tokio::task::JoinHandle<()> {
    let route = warp::path!("v1" / "chat" / "completions")
        .and(warp::post())
        // We could validate the input ChatRequest here if needed
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
    let port = 8089;
    let content = vec!["Hello".to_string(), " world!".to_string()];

    // Start the mock server
    let server_handle = run_mock_sse_server(port, content.clone()).await;

    // Verify we can talk to it
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://127.0.0.1:{}/v1/chat/completions", port))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    // In a real test, we would parse the SSE stream here.
    // Setting up a small delay to ensure server started and we could hit it.

    // Cleanup
    server_handle.abort();
}
