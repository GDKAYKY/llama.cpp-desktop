use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};

use crate::models::{
    ChatMessage, ChatRequest, LlamaCppConfig, ModelId, ModelInfo, ModelLibrary, ModelState,
    ServerMetrics,
};
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
// ==================================================================================
// 1. Actor Definition
// ==================================================================================

pub enum ActorMessage {
    Start {
        model_id: ModelId,
        config: LlamaCppConfig,
        respond_to: oneshot::Sender<Result<u32, String>>,
    },
    Stop {
        model_id: ModelId,
        respond_to: oneshot::Sender<Result<(), String>>,
    },
    SendChat {
        model_id: ModelId,
        request: ChatRequest,
        respond_to: oneshot::Sender<Result<mpsc::Receiver<String>, String>>,
    },
    IsRunning {
        model_id: Option<ModelId>,
        respond_to: oneshot::Sender<bool>,
    },
    GetConfig {
        respond_to: oneshot::Sender<Option<LlamaCppConfig>>,
    },
    GetMetrics {
        respond_to: oneshot::Sender<Option<ServerMetrics>>,
    },
    // Internal completion message
    InternalStartComplete {
        model_id: ModelId,
        result: Result<(u16, Child, LlamaCppConfig), String>,
        respond_to: oneshot::Sender<Result<u32, String>>,
    },
}

struct LlamaActor {
    registry: HashMap<ModelId, ModelInfo>,
    states: HashMap<ModelId, ModelState>,
    receiver: mpsc::Receiver<ActorMessage>,
    client: reqwest::Client,
    active_model: Option<ModelId>,
    // Store self sender for internal messages
    self_sender: mpsc::Sender<ActorMessage>,
    sys: System,
}

impl LlamaActor {
    fn new(
        receiver: mpsc::Receiver<ActorMessage>,
        self_sender: mpsc::Sender<ActorMessage>,
        initial_registry: HashMap<ModelId, ModelInfo>,
    ) -> Self {
        Self {
            registry: initial_registry,
            states: HashMap::new(),
            receiver,
            client: reqwest::Client::new(),
            active_model: None,
            self_sender,
            sys: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(ProcessRefreshKind::nothing().with_cpu())
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
            ),
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ActorMessage::Start {
                    model_id,
                    config,
                    respond_to,
                } => {
                    self.handle_start_request(model_id, config, respond_to)
                        .await;
                }
                ActorMessage::InternalStartComplete {
                    model_id,
                    result,
                    respond_to,
                } => {
                    let final_res = match result {
                        Ok((port, child, config)) => {
                            let pid = child.id().unwrap_or(0);
                            self.states.insert(
                                model_id.clone(),
                                ModelState::Running {
                                    port,
                                    child,
                                    config,
                                },
                            );
                            self.active_model = Some(model_id);
                            Ok(pid)
                        }
                        Err(e) => {
                            self.states.insert(model_id, ModelState::Stopped);
                            Err(e)
                        }
                    };
                    let _ = respond_to.send(final_res);
                }
                ActorMessage::Stop {
                    model_id,
                    respond_to,
                } => {
                    let res = self.handle_stop(&model_id).await;
                    if res.is_ok() && self.active_model.as_ref() == Some(&model_id) {
                        self.active_model = None;
                    }
                    let _ = respond_to.send(res);
                }
                ActorMessage::SendChat {
                    model_id,
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.handle_chat(&model_id, request).await);
                }
                ActorMessage::IsRunning {
                    model_id,
                    respond_to,
                } => {
                    let id_to_check = model_id.or(self.active_model.clone());
                    let running = if let Some(id) = id_to_check {
                        matches!(self.states.get(&id), Some(ModelState::Running { .. }))
                    } else {
                        false
                    };
                    let _ = respond_to.send(running);
                }
                ActorMessage::GetConfig { respond_to } => {
                    let config = if let Some(id) = &self.active_model {
                        if let Some(ModelState::Running { config, .. }) = self.states.get(id) {
                            Some(config.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let _ = respond_to.send(config);
                }
                ActorMessage::GetMetrics { respond_to } => {
                    let metrics = if let Some(id) = &self.active_model {
                        if let Some(ModelState::Running { child, .. }) = self.states.get(id) {
                            if let Some(pid) = child.id() {
                                let pid_sys = sysinfo::Pid::from(pid as usize);
                                self.sys.refresh_processes_specifics(
                                    ProcessesToUpdate::Some(&[pid_sys]),
                                    true,
                                    ProcessRefreshKind::nothing().with_cpu(),
                                );
                                if let Some(process) = self.sys.process(pid_sys) {
                                    let (gpu_usage, vram_usage) = ServerMetrics::get_gpu_metrics_for_pid(pid);
                                    Some(ServerMetrics {
                                        cpu_usage: process.cpu_usage(),
                                        mem_usage: process.memory(),
                                        gpu_usage: gpu_usage,
                                        vram_usage: vram_usage,
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let _ = respond_to.send(metrics);
                }
            }
        }
    }

    async fn handle_start_request(
        &mut self,
        model_id: ModelId,
        config: LlamaCppConfig,
        respond_to: oneshot::Sender<Result<u32, String>>,
    ) {
        // 1. Validation
        if let Some(state) = self.states.get(&model_id) {
            match state {
                ModelState::Running { child, .. } => {
                    let pid = child.id().unwrap_or(0);
                    let _ = respond_to.send(Ok(pid));
                    return;
                }
                ModelState::Starting => {
                    let _ = respond_to.send(Err(format!("Model {} is already starting", model_id)));
                    return;
                }
                _ => {}
            }
        }

        // Try to find in registry
        let mut model_entry = self.registry.get(&model_id).cloned();
        if model_entry.is_none() {
            for entry in self.registry.values() {
                if let Some(path) = &entry.model_file_path {
                    if path == &model_id.0 || Path::new(path) == Path::new(&model_id.0) {
                        model_entry = Some(entry.clone());
                        break;
                    }
                }
            }
        }

        self.states.insert(model_id.clone(), ModelState::Starting);

        // Spawn background task to do the heavy lifting
        let self_sender = self.self_sender.clone();
        let client = self.client.clone();

        tauri::async_runtime::spawn(async move {
            let result = Self::perform_start(model_entry, config.clone(), client).await;
            let _ = self_sender
                .send(ActorMessage::InternalStartComplete {
                    model_id,
                    result,
                    respond_to,
                })
                .await;
        });
    }

    async fn perform_start(
        model_entry: Option<ModelInfo>,
        config: LlamaCppConfig,
        client: reqwest::Client,
    ) -> Result<(u16, Child, LlamaCppConfig), String> {
        println!("Starting model with config: {:?}", config);
        let model_path = if let Some(entry) = model_entry {
            if let Some(path) = &entry.model_file_path {
                PathBuf::from(path)
            } else {
                PathBuf::from(&config.model_path)
            }
        } else {
            PathBuf::from(&config.model_path)
        };

        if !model_path.exists() {
            let err = format!("Model file not found: {:?}", model_path);
            println!("Error: {}", err);
            return Err(err);
        }

        let mut llama_server_path = PathBuf::from(&config.llama_cpp_path);

        // If the path is a directory, or doesn't look like the executable, try to find it
        let is_exec = llama_server_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase().contains("llama-server"))
            .unwrap_or(false);

        if !is_exec || llama_server_path.is_dir() {
            let candidates = vec!["llama-server.exe", "llama-server"];

            // Check in the provided directory
            let mut found = false;
            for candidate in &candidates {
                let p = llama_server_path.join(candidate);
                if p.exists() {
                    llama_server_path = p;
                    found = true;
                    break;
                }
            }

            // If still not found and it's a directory, maybe it's in a build subfolder
            if !found && llama_server_path.is_dir() {
                let build_candidates =
                    ["build/bin/Release/llama-server.exe", "bin/llama-server.exe"];
                for candidate in &build_candidates {
                    let p = llama_server_path.join(candidate);
                    if p.exists() {
                        llama_server_path = p;
                        found = true;
                        break;
                    }
                }
            }

            // Fallback: If still not an executable path, just join for the existence check to fail later with a good message
            if !found && !is_exec {
                let suffix = if cfg!(windows) {
                    "llama-server.exe"
                } else {
                    "llama-server"
                };
                llama_server_path = llama_server_path.join(suffix);
            }
        }

        if !llama_server_path.exists() {
            let err = format!(
                "llama-server executable not found. Checked: {:?}",
                llama_server_path
            );
            println!("Error: {}", err);
            return Err(err);
        }

        println!(
            "[LlamaActor] Spawning llama-server at: {:?} with port {}",
            llama_server_path, config.port
        );
        let binary_dir = llama_server_path.parent().unwrap_or(Path::new("."));
        let mut child = Command::new(&llama_server_path)
            .current_dir(binary_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-c")
            .arg(config.ctx_size.to_string())
            .arg("-np")
            .arg(config.parallel.to_string())
            .arg("-ngl")
            .arg(config.n_gpu_layers.to_string())
            .spawn()
            .map_err(|e| {
                let err = format!("Failed to spawn llama-server: {}", e);
                println!("Error: {}", err);
                err
            })?;

        if let Some(stdout) = child.stdout.take() {
            let mut reader = tokio::io::BufReader::new(stdout);
            tauri::async_runtime::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    print!("[llama-server] {}", line);
                    line.clear();
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = tokio::io::BufReader::new(stderr);
            tauri::async_runtime::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    eprint!("[llama-server] {}", line);
                    line.clear();
                }
            });
        }

        // Healthcheck
        let port = config.port;
        let health_url = format!("http://localhost:{}/health", port);
        let mut attempts = 0;
        let max_attempts = 40; // 20 seconds
        let mut success = false;

        while attempts < max_attempts {
            // Check if process died
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("llama-server exited early with status: {}", status));
            }

            match client
                .get(&health_url)
                .timeout(Duration::from_secs(1))
                .send()
                .await
            {
                Ok(res) if res.status().is_success() => {
                    success = true;
                    break;
                }
                _ => {}
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }

        if success {
            Ok((port, child, config))
        } else {
            let _ = child.kill().await;
            Err("Failed to start model: Healthcheck timed out".to_string())
        }
    }

    async fn handle_stop(&mut self, model_id: &ModelId) -> Result<(), String> {
        if let Some(state) = self.states.remove(model_id) {
            if let ModelState::Running { mut child, .. } = state {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
            self.states.insert(model_id.clone(), ModelState::Stopped);
        }
        Ok(())
    }

    async fn handle_chat(
        &mut self,
        model_id: &ModelId,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<String>, String> {
        let port = if let Some(ModelState::Running { port, .. }) = self.states.get(model_id) {
            *port
        } else {
            return Err(format!("Model {} is not running", model_id));
        };

        let url = format!("http://localhost:{}/v1/chat/completions", port);
        let (tx, rx) = mpsc::channel(32);
        let client = self.client.clone();

        tauri::async_runtime::spawn(async move {
            println!(
                "[LlamaActor] Sending Request to llama-server: {}",
                serde_json::to_string_pretty(&request).unwrap()
            );
            let res = client
                .post(&url)
                .json(&request)
                .timeout(Duration::from_secs(300))
                .send()
                .await;
            match res {
                Ok(mut response) => {
                    if !response.status().is_success() {
                        let _ = tx
                            .send(format!("Error: Status {}", response.status()))
                            .await;
                        return;
                    }
                    let mut buffer = String::new();
                    while let Ok(Some(chunk)) = response.chunk().await {
                        let s = String::from_utf8_lossy(&chunk);
                        // println!("[LlamaActor] Received chunk: {}", s);
                        buffer.push_str(&s);

                        while let Some(idx) = buffer.find('\n') {
                            let line = buffer[..idx].trim().to_string();
                            buffer = buffer[idx + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            if line.starts_with("data:") {
                                let data_content = line["data:".len()..].trim();

                                if data_content == "[DONE]" {
                                    println!("[LlamaActor] Stream signaled [DONE]");
                                    return;
                                }

                                if let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(data_content)
                                {
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        if tx.send(content.to_string()).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            } else if line.starts_with(":") {
                                // SSE Comment/keep-alive, ignore
                            } else {
                                // Try to parse as raw JSON in case it's not strictly SSE
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        if tx.send(content.to_string()).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Error: Connect {}", e)).await;
                }
            }
        });
        Ok(rx)
    }
}

// ==================================================================================
// 3. Public Service Interface
// ==================================================================================

#[derive(Clone)]
pub struct LlamaCppService {
    sender: mpsc::Sender<ActorMessage>,
}

impl LlamaCppService {
    pub fn new(models_path: PathBuf) -> Self {
        let registry_path = models_path.join("modelLibrary.json");
        let initial_registry = if registry_path.exists() {
            if let Ok(file) = std::fs::File::open(&registry_path) {
                let reader = std::io::BufReader::new(file);
                if let Ok(lib) = serde_json::from_reader::<_, ModelLibrary>(reader) {
                    lib.models
                        .into_iter()
                        .map(|m| (ModelId(m.full_identifier.clone()), m))
                        .collect()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        let (tx, rx) = mpsc::channel(64);
        let tx_clone = tx.clone();
        let mut actor = LlamaActor::new(rx, tx_clone, initial_registry);

        tauri::async_runtime::spawn(async move {
            actor.run().await;
        });

        Self { sender: tx }
    }

    pub async fn start(&self, config: LlamaCppConfig) -> Result<u32, String> {
        let id = ModelId(config.model_path.clone());
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(ActorMessage::Start {
                model_id: id,
                config,
                respond_to: tx,
            })
            .await
            .map_err(|e| e.to_string())?;

        rx.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn stop(&self) -> Result<(), String> {
        if let Some(config) = self.get_config().await {
            let id = ModelId(config.model_path);
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ActorMessage::Stop {
                    model_id: id,
                    respond_to: tx,
                })
                .await
                .map_err(|e| e.to_string())?;
            return rx.await.map_err(|_| "Actor dropped".to_string())?;
        }
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::IsRunning {
                model_id: None,
                respond_to: tx,
            })
            .await;
        rx.await.unwrap_or(false)
    }

    pub async fn send_chat_message(
        &self,
        session_id: Option<String>,
        messages: Vec<ChatMessage>,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        max_tokens: i32,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, String> {
        let config = self.get_config().await.ok_or("No model running")?;
        let id = ModelId(config.model_path);
        let request = ChatRequest {
            model: "unknown".to_string(),
            session_id,
            messages,
            temperature,
            top_p,
            top_k,
            max_tokens,
            stream: true,
        };
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(ActorMessage::SendChat {
                model_id: id,
                request,
                respond_to: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn get_config(&self) -> Option<LlamaCppConfig> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::GetConfig { respond_to: tx })
            .await;
        rx.await.unwrap_or(None)
    }

    pub async fn get_metrics(&self) -> Option<ServerMetrics> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(ActorMessage::GetMetrics { respond_to: tx })
            .await;
        rx.await.unwrap_or(None)
    }
}
