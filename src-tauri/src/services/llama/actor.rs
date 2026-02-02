use std::collections::HashMap;
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tokio::sync::{mpsc, oneshot};

use crate::infrastructure::llama::server::LlamaServer;
use crate::infrastructure::nvidia_smi::NvidiaSmi;
use crate::models::{ChatRequest, LlamaCppConfig, ModelId, ModelInfo, ModelState, ServerMetrics};

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
    InternalStartComplete {
        model_id: ModelId,
        result: Result<(u16, tokio::process::Child), String>,
        config: LlamaCppConfig,
        respond_to: oneshot::Sender<Result<u32, String>>,
    },
}

pub struct LlamaActor {
    registry: HashMap<ModelId, ModelInfo>,
    states: HashMap<ModelId, ModelState>,
    receiver: mpsc::Receiver<ActorMessage>,
    client: reqwest::Client,
    active_model: Option<ModelId>,
    self_sender: mpsc::Sender<ActorMessage>,
    sys: System,
}

impl LlamaActor {
    pub fn new(
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

    pub async fn run(&mut self) {
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
                    config,
                    respond_to,
                } => {
                    let final_res = match result {
                        Ok((port, child)) => {
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
                    let config = self
                        .active_model
                        .as_ref()
                        .and_then(|id| self.states.get(id))
                        .and_then(|state| match state {
                            ModelState::Running { config, .. } => Some(config.clone()),
                            _ => None,
                        });
                    let _ = respond_to.send(config);
                }
                ActorMessage::GetMetrics { respond_to } => {
                    let metrics = self.handle_get_metrics().await;
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
        if let Some(state) = self.states.get(&model_id) {
            match state {
                ModelState::Running { child, .. } => {
                    let _ = respond_to.send(Ok(child.id().unwrap_or(0)));
                    return;
                }
                ModelState::Starting => {
                    let _ = respond_to.send(Err(format!("Model {} is already starting", model_id)));
                    return;
                }
                _ => {}
            }
        }

        let model_entry = self.registry.get(&model_id).cloned();
        self.states.insert(model_id.clone(), ModelState::Starting);

        let self_sender = self.self_sender.clone();
        let client = self.client.clone();
        let config_clone = config.clone();

        tauri::async_runtime::spawn(async move {
            let result = LlamaServer::spawn(model_entry, config_clone.clone(), client).await;
            let _ = self_sender
                .send(ActorMessage::InternalStartComplete {
                    model_id,
                    result,
                    config: config_clone,
                    respond_to,
                })
                .await;
        });
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

        LlamaServer::stream_chat(self.client.clone(), port, request).await
    }

    async fn handle_get_metrics(&mut self) -> Option<ServerMetrics> {
        let id = self.active_model.as_ref()?;
        let (pid, child_id) = match self.states.get(id)? {
            ModelState::Running { child, .. } => {
                let pid = child.id()?;
                (sysinfo::Pid::from(pid as usize), pid)
            }
            _ => return None,
        };

        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );

        let process = self.sys.process(pid)?;
        let (gpu_usage, vram_usage) = NvidiaSmi::get_gpu_metrics_for_pid(child_id);

        Some(ServerMetrics {
            cpu_usage: process.cpu_usage(),
            mem_usage: process.memory(),
            gpu_usage,
            vram_usage,
        })
    }
}
