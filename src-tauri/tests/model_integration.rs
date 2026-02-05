use llama_desktop_lib::models::{ChatMessage, LlamaCppConfig};
use llama_desktop_lib::services::llama::LlamaCppService;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

/// Helper function to find a random available port on localhost.
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a dynamic port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

#[tokio::test]
async fn test_actual_model_call() {
    // Explicit opt-in for this slow integration test.
    if std::env::var("RUN_LLM_INTEGRATION").ok().as_deref() != Some("1") {
        println!("SKIPPING TEST: RUN_LLM_INTEGRATION not set to 1.");
        return;
    }
    let testname = "test_actual_model_call";
    // Configuration via environment variables (CI-friendly).
    let llama_cpp_path = match std::env::var("LLAMA_CPP_PATH") {
        Ok(path) => path,
        Err(_) => {
            println!("SKIPPING TEST: LLAMA_CPP_PATH not set.");
            return;
        }
    };
    let model_path = match std::env::var("LLAMA_MODEL_PATH") {
        Ok(path) => path,
        Err(_) => {
            println!("SKIPPING TEST: LLAMA_MODEL_PATH not set.");
            return;
        }
    };
    let models_root = PathBuf::from(std::env::var("LLAMA_MODELS_ROOT").unwrap_or_else(|_| ".".to_string()));

    if !PathBuf::from(&llama_cpp_path).exists() || !PathBuf::from(&model_path).exists() {
        println!(
            "SKIPPING TEST: Required paths not found. Set LLAMA_CPP_PATH and LLAMA_MODEL_PATH."
        );
        return;
    }

    let port = get_available_port();
    let service = LlamaCppService::new(models_root);

    let config = LlamaCppConfig {
        llama_cpp_path,
        model_path: model_path.clone(),
        port,
        ctx_size: 2048,
        parallel: 1,
        n_gpu_layers: 0, // Using CPU for predictable test environments
    };

    println!("--- [1/3] Starting Llama Server on port {} ---", port);

    // START TIMEOUT: Prevent test from hanging if server fails to initialize
    let start_res = timeout(Duration::from_secs(45), service.start(config))
        .await
        .expect("Llama server start timed out after 45s");

    let pid = start_res.expect("Failed to start llama-server");
    println!("Server started with PID: {}", pid);

    println!("--- [2/3] Sending Chat Message & Checking Semantic Response ---");

    let magic_token = "RUST_INTEGRATION_SUCCESS";
    let prompt = format!("Say exactly this word and nothing else: {}", magic_token);

    // CHAT TIMEOUT: Wrap the entire communication logic
    let chat_task = async {
        let mut rx: tokio::sync::mpsc::Receiver<String> = service
            .send_chat_message(
                Some("integration-test-session".to_string()),
                vec![ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                0.1, // Low temperature for deterministic output
                0.95,
                40,
                128,
            )
            .await
            .expect("Failed to send chat message");

        let mut full_response = String::new();
        while let Some(chunk) = rx.recv().await {
            print!("{}", chunk);
            full_response.push_str(&chunk);
        }
        full_response
    };

    let full_response = timeout(Duration::from_secs(60), chat_task)
        .await
        .expect("Model communication timed out after 60s");

    println!("\n--- [3/3] Shutting Down ---");

    // CLEANUP: Ensure stop is called before assertions that might panic
    let stop_res = timeout(Duration::from_secs(10), service.stop())
        .await
        .expect("Stop timed out after 10s");

    // SEMANTIC ASSERTION
    assert!(
        full_response.contains(magic_token),
        "Model failed semantic check. Expected token '{}' in response: '{}'",
        magic_token,
        full_response
    );

    stop_res.expect("Failed to stop server cleanly");
    println!("\x1b[30;42m SUCESS \x1b[0m {}", testname);
}
