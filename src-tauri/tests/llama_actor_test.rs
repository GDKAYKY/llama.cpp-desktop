use llama_desktop_lib::infrastructure::llama::process::LlamaProcessManager;
use llama_desktop_lib::infrastructure::metrics::MetricsProvider;
use llama_desktop_lib::models::{
    ChatRequest, LlamaCppConfig, ManifestConfig, ManifestLayer, ModelId, ModelInfo, ModelManifest,
    ModelState, ServerMetrics,
};
use llama_desktop_lib::services::llama::actor::{ActorMessage, LlamaActor};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::{mpsc, oneshot};
use warp::Filter;

struct MockMetricsProvider;

impl MetricsProvider for MockMetricsProvider {
    fn snapshot_for_pid(&self, pid: u32) -> Option<ServerMetrics> {
        Some(ServerMetrics {
            cpu_usage: pid as f32,
            mem_usage: 10,
            gpu_usage: None,
            vram_usage: None,
        })
    }
}

async fn spawn_sleep_child() -> tokio::process::Child {
    #[cfg(windows)]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(["/C", "ping", "127.0.0.1", "-n", "6"]);
        cmd
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("sleep");
        cmd.arg("5");
        cmd
    };
    cmd.spawn().expect("spawn child")
}

fn write_dummy_model(dir: &std::path::Path) -> PathBuf {
    let model_path = dir.join("model.gguf");
    std::fs::write(&model_path, "model").expect("write model");
    model_path
}

fn write_dummy_llama_server(dir: &std::path::Path) -> PathBuf {
    #[cfg(windows)]
    let path = dir.join("llama-server.cmd");
    #[cfg(not(windows))]
    let path = dir.join("llama-server");

    #[cfg(windows)]
    let content = "@echo off\r\nping 127.0.0.1 -n 60 >nul\r\n".to_string();
    #[cfg(not(windows))]
    let content = "#!/usr/bin/env sh\nsleep 60\n".to_string();

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
    let (addr, server) =
        warp::serve(route).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async move {
            let _ = shutdown_rx.await;
        });
    tokio::spawn(server);
    (addr, shutdown_tx)
}

fn sample_manifest(model_path: String) -> ModelInfo {
    let manifest = ModelManifest {
        schema_version: 2,
        media_type: "application/vnd.ollama.manifest.v1+json".to_string(),
        config: ManifestConfig {
            media_type: "application/vnd.ollama.image.config".to_string(),
            digest: "sha256:config".to_string(),
            size: 1,
        },
        layers: vec![ManifestLayer {
            media_type: "application/vnd.ollama.image.model".to_string(),
            digest: "sha256:model".to_string(),
            size: 1,
        }],
    };

    ModelInfo {
        provider: "provider".to_string(),
        library: "lib".to_string(),
        name: "name".to_string(),
        version: "v1".to_string(),
        manifest,
        model_file_path: Some(model_path),
        full_identifier: "provider:name:v1".to_string(),
    }
}

#[tokio::test]
async fn internal_start_complete_updates_state() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let model_id = ModelId("model-1".to_string());
    let config = LlamaCppConfig {
        llama_cpp_path: "llama".to_string(),
        model_path: "model".to_string(),
        port: 8080,
        ctx_size: 128,
        parallel: 1,
        n_gpu_layers: 0,
    };
    let child = spawn_sleep_child().await;
    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(ActorMessage::InternalStartComplete {
        model_id: model_id.clone(),
        result: Ok((8080, child)),
        config: config.clone(),
        respond_to: resp_tx,
    })
    .await
    .expect("send");
    let pid = resp_rx.await.expect("resp").expect("ok");
    assert!(pid > 0);

    let (cfg_tx, cfg_rx) = oneshot::channel();
    tx.send(ActorMessage::GetConfig { respond_to: cfg_tx })
        .await
        .expect("send");
    let cfg = cfg_rx.await.expect("cfg");
    assert_eq!(cfg.unwrap().model_path, config.model_path);
}

#[tokio::test]
async fn internal_start_complete_error_sets_stopped() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(ActorMessage::InternalStartComplete {
        model_id: ModelId("model-2".to_string()),
        result: Err("boom".to_string()),
        config: LlamaCppConfig {
            llama_cpp_path: "llama".to_string(),
            model_path: "model".to_string(),
            port: 8080,
            ctx_size: 128,
            parallel: 1,
            n_gpu_layers: 0,
        },
        respond_to: resp_tx,
    })
    .await
    .expect("send");
    let err = resp_rx.await.expect("resp").expect_err("err");
    assert!(err.contains("boom"));
}

#[tokio::test]
async fn handle_chat_returns_error_when_not_running() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );

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
    let err = actor
        .test_handle_chat(&ModelId("model".to_string()), request)
        .await
        .expect_err("expected error");
    assert!(err.contains("not running"));
}

#[tokio::test]
async fn get_metrics_returns_none_without_active_model() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    let metrics = actor.test_handle_get_metrics().await;
    assert!(metrics.is_none());
}

#[tokio::test]
async fn get_metrics_returns_none_for_zero_pid() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    let model_id = ModelId("model".to_string());
    actor.test_set_active_model(Some(model_id.clone()));
    actor.test_set_state(
        model_id,
        ModelState::Running {
            port: 0,
            pid: 0,
            config: LlamaCppConfig {
                llama_cpp_path: "llama".to_string(),
                model_path: "model".to_string(),
                port: 0,
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
        },
    );
    let metrics = actor.test_handle_get_metrics().await;
    assert!(metrics.is_none());
}

#[tokio::test]
async fn get_metrics_returns_none_for_non_running_state() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    let model_id = ModelId("model".to_string());
    actor.test_set_active_model(Some(model_id.clone()));
    actor.test_set_state(model_id, ModelState::Stopped);
    let metrics = actor.test_handle_get_metrics().await;
    assert!(metrics.is_none());
}

#[tokio::test]
async fn get_model_lock_reuses_existing_lock() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    let model_id = ModelId("model".to_string());
    let first = actor.test_get_model_lock(&model_id);
    let second = actor.test_get_model_lock(&model_id);
    assert!(Arc::ptr_eq(&first, &second));
}

#[tokio::test]
async fn handle_start_request_returns_running_pid() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );

    let model_id = ModelId("model-1".to_string());
    actor.test_set_state(
        model_id.clone(),
        ModelState::Running {
            port: 8080,
            pid: 123,
            config: LlamaCppConfig {
                llama_cpp_path: "llama".to_string(),
                model_path: "model".to_string(),
                port: 8080,
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
        },
    );

    let (resp_tx, resp_rx) = oneshot::channel();
    actor
        .test_handle_start_request(
            model_id,
            LlamaCppConfig {
                llama_cpp_path: "llama".to_string(),
                model_path: "model".to_string(),
                port: 8080,
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
            resp_tx,
        )
        .await;
    let pid = resp_rx.await.expect("resp").expect("ok");
    assert_eq!(pid, 123);
}

#[tokio::test]
async fn handle_start_request_returns_starting_error() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx,
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );

    let model_id = ModelId("model-1".to_string());
    actor.test_set_state(model_id.clone(), ModelState::Starting);

    let (resp_tx, resp_rx) = oneshot::channel();
    actor
        .test_handle_start_request(
            model_id.clone(),
            LlamaCppConfig {
                llama_cpp_path: "llama".to_string(),
                model_path: "model".to_string(),
                port: 8080,
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
            resp_tx,
        )
        .await;
    let err = resp_rx.await.expect("resp").expect_err("err");
    assert!(err.contains("already starting"));
}

#[tokio::test]
async fn handle_start_request_spawns_server_and_updates_state() {
    let dir = tempdir().expect("tempdir");
    let model_path = write_dummy_model(dir.path());
    let server_path = write_dummy_llama_server(dir.path());
    let (addr, shutdown_tx) = start_health_server().await;

    let model_info = sample_manifest(model_path.to_string_lossy().to_string());
    let mut registry = HashMap::new();
    let model_id = ModelId(model_info.full_identifier.clone());
    registry.insert(model_id.clone(), model_info);

    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        registry,
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );

    let (resp_tx, resp_rx) = oneshot::channel();
    actor
        .test_handle_start_request(
            model_id.clone(),
            LlamaCppConfig {
                llama_cpp_path: server_path.to_string_lossy().to_string(),
                model_path: model_path.to_string_lossy().to_string(),
                port: addr.port(),
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
            resp_tx,
        )
        .await;

    let actor_task = tokio::spawn(async move { actor.run().await });

    let pid = resp_rx.await.expect("resp").expect("ok");
    assert!(pid > 0);

    let (run_tx, run_rx) = oneshot::channel();
    tx.send(ActorMessage::IsRunning {
        model_id: Some(model_id.clone()),
        respond_to: run_tx,
    })
    .await
    .expect("send");
    assert!(run_rx.await.expect("running"));

    let (stop_tx, stop_rx) = oneshot::channel();
    tx.send(ActorMessage::Stop {
        model_id,
        respond_to: stop_tx,
    })
    .await
    .expect("stop");
    let _ = stop_rx.await.expect("stop ok");

    let _ = shutdown_tx.send(());
    actor_task.abort();
}

#[tokio::test]
async fn stop_clears_active_model() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let model_id = ModelId("model-3".to_string());
    let child = spawn_sleep_child().await;
    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(ActorMessage::InternalStartComplete {
        model_id: model_id.clone(),
        result: Ok((8081, child)),
        config: LlamaCppConfig {
            llama_cpp_path: "llama".to_string(),
            model_path: "model".to_string(),
            port: 8081,
            ctx_size: 128,
            parallel: 1,
            n_gpu_layers: 0,
        },
        respond_to: resp_tx,
    })
    .await
    .expect("send");
    let _ = resp_rx.await.expect("resp");

    let (stop_tx, stop_rx) = oneshot::channel();
    tx.send(ActorMessage::Stop {
        model_id: model_id.clone(),
        respond_to: stop_tx,
    })
    .await
    .expect("stop");
    let _ = stop_rx.await.expect("stop ok");

    let (run_tx, run_rx) = oneshot::channel();
    tx.send(ActorMessage::IsRunning {
        model_id: Some(model_id),
        respond_to: run_tx,
    })
    .await
    .expect("send");
    assert!(!run_rx.await.expect("running"));
}

#[tokio::test]
async fn stop_handles_missing_state() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let (stop_tx, stop_rx) = oneshot::channel();
    tx.send(ActorMessage::Stop {
        model_id: ModelId("missing".to_string()),
        respond_to: stop_tx,
    })
    .await
    .expect("send");
    stop_rx.await.expect("stop").expect("ok");
}

#[tokio::test]
async fn is_running_false_when_no_active_model() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let (run_tx, run_rx) = oneshot::channel();
    tx.send(ActorMessage::IsRunning {
        model_id: None,
        respond_to: run_tx,
    })
    .await
    .expect("send");
    assert!(!run_rx.await.expect("running"));
}

#[tokio::test]
async fn get_config_returns_none_without_active_model() {
    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    tokio::spawn(async move { actor.run().await });

    let (cfg_tx, cfg_rx) = oneshot::channel();
    tx.send(ActorMessage::GetConfig { respond_to: cfg_tx })
        .await
        .expect("send");
    assert!(cfg_rx.await.expect("cfg").is_none());
}

#[tokio::test]
async fn send_chat_streams_chunks() {
    let route = warp::path("v1")
        .and(warp::path("chat"))
        .and(warp::path("completions"))
        .and(warp::post())
        .map(|| {
            let body = warp::hyper::Body::from(
                "data: {\"choices\":[{\"delta\":{\"content\":\"A\"}}]}\n\ndata: [DONE]\n",
            );
            warp::reply::Response::new(body)
        });
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);

    let (tx, rx) = mpsc::channel(8);
    let mut actor = LlamaActor::new(
        rx,
        tx.clone(),
        HashMap::new(),
        Arc::new(LlamaProcessManager::new()),
        Arc::new(MockMetricsProvider),
    );
    actor.test_set_state(
        ModelId("model".to_string()),
        ModelState::Running {
            port: addr.port(),
            pid: 1,
            config: LlamaCppConfig {
                llama_cpp_path: "llama".to_string(),
                model_path: "model".to_string(),
                port: addr.port(),
                ctx_size: 128,
                parallel: 1,
                n_gpu_layers: 0,
            },
        },
    );
    tokio::spawn(async move { actor.run().await });

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
    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(ActorMessage::SendChat {
        model_id: ModelId("model".to_string()),
        request,
        respond_to: resp_tx,
    })
    .await
    .expect("send");
    let mut rx = resp_rx.await.expect("resp").expect("ok");
    let chunk = rx.recv().await.expect("chunk");
    assert_eq!(chunk, "A");
}
