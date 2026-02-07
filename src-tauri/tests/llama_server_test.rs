use async_stream::stream;
use llama_desktop_lib::infrastructure::llama::server::LlamaServer;
use llama_desktop_lib::models::{ChatRequest, LlamaCppConfig};
use tempfile::tempdir;
use tokio::sync::oneshot;
use warp::hyper::body::Bytes;
use warp::Filter;

fn write_dummy_model(dir: &std::path::Path) -> std::path::PathBuf {
    let model_path = dir.join("model.gguf");
    std::fs::write(&model_path, "model").expect("write model");
    model_path
}

fn write_dummy_llama_server(
    dir: &std::path::Path,
    exit_immediately: bool,
) -> std::path::PathBuf {
    #[cfg(windows)]
    let path = dir.join("llama-server.cmd");
    #[cfg(not(windows))]
    let path = dir.join("llama-server");

    #[cfg(windows)]
    let content = if exit_immediately {
        "@echo off\r\nexit /b 1\r\n".to_string()
    } else {
        "@echo off\r\nping 127.0.0.1 -n 60 >nul\r\n".to_string()
    };
    #[cfg(not(windows))]
    let content = if exit_immediately {
        "#!/usr/bin/env sh\nexit 1\n".to_string()
    } else {
        "#!/usr/bin/env sh\nsleep 60\n".to_string()
    };

    std::fs::write(&path, content).expect("write server");
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod");
    }
    path
}

async fn start_health_server() -> (std::net::SocketAddr, oneshot::Sender<()>) {
    let route = warp::path("health").map(|| "ok");
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (addr, server) = warp::serve(route).bind_with_graceful_shutdown(
        ([127, 0, 0, 1], 0),
        async move {
            let _ = shutdown_rx.await;
        },
    );
    tokio::spawn(server);
    (addr, shutdown_tx)
}

#[tokio::test]
async fn pipe_output_handles_missing_streams() {
    #[cfg(windows)]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(["/C", "ping", "127.0.0.1", "-n", "2"]);
        cmd
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("sleep");
        cmd.arg("1");
        cmd
    };
    let mut child = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn");
    LlamaServer::test_pipe_output(&mut child);
    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[tokio::test]
async fn spawn_errors_when_model_missing() {
    let dir = tempdir().expect("tempdir");
    let dummy_server = write_dummy_llama_server(dir.path(), false);
    let config = LlamaCppConfig {
        llama_cpp_path: dummy_server.to_string_lossy().to_string(),
        model_path: dir.path().join("missing.gguf").to_string_lossy().to_string(),
        port: 7777,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let err = LlamaServer::spawn(None, config, client)
        .await
        .expect_err("expected error");
    assert!(err.contains("Model file not found"));
}

#[tokio::test]
async fn spawn_errors_when_server_missing() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let config = LlamaCppConfig {
        llama_cpp_path: dir.path().to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: 7777,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let err = LlamaServer::spawn(None, config, client)
        .await
        .expect_err("expected error");
    assert!(err.contains("llama-server executable not found"));
}

#[tokio::test]
async fn spawn_errors_when_process_exits_early() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let dummy_server = write_dummy_llama_server(dir.path(), true);
    let config = LlamaCppConfig {
        llama_cpp_path: dummy_server.to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: 7777,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let err = LlamaServer::spawn(None, config, client)
        .await
        .expect_err("expected error");
    assert!(err.contains("llama-server exited early"));
}

#[tokio::test]
async fn spawn_errors_on_healthcheck_timeout() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let dummy_server = write_dummy_llama_server(dir.path(), false);
    let config = LlamaCppConfig {
        llama_cpp_path: dummy_server.to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: 6553,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let err = LlamaServer::spawn(None, config, client)
        .await
        .expect_err("expected error");
    assert!(err.contains("Healthcheck timed out"));
}

#[tokio::test]
async fn spawn_success_with_dummy_server_and_healthcheck() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let dummy_server = write_dummy_llama_server(dir.path(), false);
    let (addr, shutdown_tx) = start_health_server().await;

    let config = LlamaCppConfig {
        llama_cpp_path: dummy_server.to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: addr.port(),
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let (_port, mut child) = LlamaServer::spawn(None, config, client)
        .await
        .expect("spawn ok");
    let _ = child.kill().await;
    let _ = child.wait().await;
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn spawn_finds_server_in_directory_candidates() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let server_path = write_dummy_llama_server(dir.path(), false);
    let (addr, shutdown_tx) = start_health_server().await;

    let config = LlamaCppConfig {
        llama_cpp_path: dir.path().to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: addr.port(),
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let (_port, mut child) = LlamaServer::spawn(None, config, client)
        .await
        .expect("spawn ok");
    let _ = child.kill().await;
    let _ = child.wait().await;
    let _ = shutdown_tx.send(());

    assert!(server_path.exists());
}

#[tokio::test]
async fn spawn_finds_server_in_build_candidates() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let build_dir = dir.path().join("bin");
    std::fs::create_dir_all(&build_dir).expect("create bin");
    #[cfg(windows)]
    let candidate = build_dir.join("llama-server.cmd");
    #[cfg(not(windows))]
    let candidate = build_dir.join("llama-server");
    #[cfg(windows)]
    let content = "@echo off\r\nping 127.0.0.1 -n 60 >nul\r\n".to_string();
    #[cfg(not(windows))]
    let content = "#!/usr/bin/env sh\nsleep 60\n".to_string();
    std::fs::write(&candidate, content).expect("write candidate");
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&candidate).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&candidate, perms).expect("chmod");
    }

    let (addr, shutdown_tx) = start_health_server().await;
    let config = LlamaCppConfig {
        llama_cpp_path: dir.path().to_string_lossy().to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        port: addr.port(),
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let client = reqwest::Client::new();
    let (_port, mut child) = LlamaServer::spawn(None, config, client)
        .await
        .expect("spawn ok");
    let _ = child.kill().await;
    let _ = child.wait().await;
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn stream_chat_parses_data_and_json_lines() {
    let route = warp::path("v1")
        .and(warp::path("chat"))
        .and(warp::path("completions"))
        .and(warp::post())
        .map(|| {
            let stream = stream! {
                let payload = concat!(
                    ": keepalive\n",
                    "data: {bad json}\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n",
                    "\n",
                    "not json\n",
                    "{\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n",
                    "data: [DONE]\n"
                );
                yield Ok::<Bytes, std::io::Error>(Bytes::from(payload));
            };
            let body = warp::hyper::Body::wrap_stream(stream);
            warp::reply::Response::new(body)
        });

    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let request = ChatRequest {
        model: "test".to_string(),
        session_id: None,
        messages: vec![],
        temperature: 0.1,
        top_p: 1.0,
        top_k: 1,
        max_tokens: 1,
        stream: true,
    };

    let mut rx = LlamaServer::stream_chat(reqwest::Client::new(), addr.port(), request)
        .await
        .expect("stream chat");
    let mut collected = String::new();
    while let Some(chunk) = rx.recv().await {
        collected.push_str(&chunk);
    }
    assert_eq!(collected, "Hello");
}

#[tokio::test]
async fn stream_chat_emits_error_on_non_success() {
    let route = warp::path("v1")
        .and(warp::path("chat"))
        .and(warp::path("completions"))
        .and(warp::post())
        .map(|| warp::reply::with_status("nope", warp::http::StatusCode::BAD_REQUEST));
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let request = ChatRequest {
        model: "test".to_string(),
        session_id: None,
        messages: vec![],
        temperature: 0.1,
        top_p: 1.0,
        top_k: 1,
        max_tokens: 1,
        stream: true,
    };

    let mut rx = LlamaServer::stream_chat(reqwest::Client::new(), addr.port(), request)
        .await
        .expect("stream chat");
    let first = rx.recv().await.expect("error message");
    assert!(first.contains("Status"));
}
