use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};

use crate::infrastructure::llama::process::ProcessManager;
use crate::infrastructure::llama::server::LlamaServer;
use crate::infrastructure::metrics::MetricsProvider;
use crate::models::{
    ActiveModel, ChatRequest, LlamaCppConfig, ModelId, ModelInfo, ModelState, ServerMetrics,
};

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
    CompleteChat {
        model_id: ModelId,
        request: ChatRequest,
        respond_to: oneshot::Sender<Result<serde_json::Value, String>>,
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
    active_model: ActiveModel,
    self_sender: mpsc::Sender<ActorMessage>,
    process_manager: Arc<dyn ProcessManager>,
    metrics: Arc<dyn MetricsProvider>,
    model_locks: HashMap<ModelId, Arc<TokioMutex<()>>>,
}

impl LlamaActor {
    pub fn new(
        receiver: mpsc::Receiver<ActorMessage>,
        self_sender: mpsc::Sender<ActorMessage>,
        initial_registry: HashMap<ModelId, ModelInfo>,
        process_manager: Arc<dyn ProcessManager>,
        metrics: Arc<dyn MetricsProvider>,
    ) -> Self {
        Self {
            registry: initial_registry,
            states: HashMap::new(),
            receiver,
            client: reqwest::Client::new(),
            active_model: None,
            self_sender,
            process_manager,
            metrics,
            model_locks: HashMap::new(),
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
                    let lock = self.get_model_lock(&model_id);
                    let _guard = lock.lock().await;
                    let final_res = match result {
                        Ok((port, child)) => {
                            let pid = child.id().unwrap_or(0);
                            self.process_manager.register(model_id.clone(), child);
                            self.states.insert(
                                model_id.clone(),
                                ModelState::Running { port, pid, config },
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
                ActorMessage::CompleteChat {
                    model_id,
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.handle_complete_chat(&model_id, request).await);
                }
                ActorMessage::IsRunning {
                    model_id,
                    respond_to,
                } => {
                    let id_to_check = model_id.or(self.active_model.clone());
                    let running = if let Some(id) = id_to_check {
                        let lock = self.get_model_lock(&id);
                        let _guard = lock.lock().await;
                        matches!(self.states.get(&id), Some(ModelState::Running { .. }))
                    } else {
                        false
                    };
                    let _ = respond_to.send(running);
                }
                ActorMessage::GetConfig { respond_to } => {
                    let config = if let Some(id) = self.active_model.clone() {
                        let lock = self.get_model_lock(&id);
                        let _guard = lock.lock().await;
                        self.states.get(&id).and_then(|state| match state {
                            ModelState::Running { config, .. } => Some(config.clone()),
                            _ => None,
                        })
                    } else {
                        None
                    };
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
        let lock = self.get_model_lock(&model_id);
        let _guard = lock.lock().await;
        if let Some(state) = self.states.get(&model_id) {
            match state {
                ModelState::Running { pid, .. } => {
                    let _ = respond_to.send(Ok(*pid));
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
        drop(_guard);

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
        let lock = self.get_model_lock(model_id);
        let _guard = lock.lock().await;
        let child = self.process_manager.remove(model_id);
        if let Some(_state) = self.states.remove(model_id) {
            self.states.insert(model_id.clone(), ModelState::Stopped);
        }
        drop(_guard);
        if let Some(mut child) = child {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        Ok(())
    }

    async fn handle_chat(
        &mut self,
        model_id: &ModelId,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<String>, String> {
        let lock = self.get_model_lock(model_id);
        let _guard = lock.lock().await;
        let port = if let Some(ModelState::Running { port, .. }) = self.states.get(model_id) {
            *port
        } else {
            return Err(format!("Model {} is not running", model_id));
        };

        LlamaServer::stream_chat(self.client.clone(), port, request).await
    }

    async fn handle_complete_chat(
        &mut self,
        model_id: &ModelId,
        request: ChatRequest,
    ) -> Result<serde_json::Value, String> {
        let lock = self.get_model_lock(model_id);
        let _guard = lock.lock().await;
        let port = if let Some(ModelState::Running { port, .. }) = self.states.get(model_id) {
            *port
        } else {
            return Err(format!("Model {} is not running", model_id));
        };

        LlamaServer::chat_completion(self.client.clone(), port, request).await
    }

    async fn handle_get_metrics(&mut self) -> Option<ServerMetrics> {
        let id = self.active_model.clone()?;
        let lock = self.get_model_lock(&id);
        let _guard = lock.lock().await;
        let pid = match self.states.get(&id)? {
            ModelState::Running { pid, .. } => *pid,
            _ => return None,
        };
        drop(_guard);
        if pid == 0 {
            return None;
        }
        self.metrics.snapshot_for_pid(pid)
    }

    fn get_model_lock(&mut self, model_id: &ModelId) -> Arc<TokioMutex<()>> {
        self.model_locks
            .entry(model_id.clone())
            .or_insert_with(|| Arc::new(TokioMutex::new(())))
            .clone()
    }
}

impl LlamaActor {
    pub async fn test_handle_chat(
        &mut self,
        model_id: &ModelId,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<String>, String> {
        self.handle_chat(model_id, request).await
    }

    pub async fn test_handle_get_metrics(&mut self) -> Option<ServerMetrics> {
        self.handle_get_metrics().await
    }

    pub fn test_get_model_lock(&mut self, model_id: &ModelId) -> Arc<TokioMutex<()>> {
        self.get_model_lock(model_id)
    }

    pub async fn test_handle_start_request(
        &mut self,
        model_id: ModelId,
        config: LlamaCppConfig,
        respond_to: oneshot::Sender<Result<u32, String>>,
    ) {
        self.handle_start_request(model_id, config, respond_to).await;
    }

    pub fn test_set_state(&mut self, model_id: ModelId, state: ModelState) {
        self.states.insert(model_id, state);
    }

    pub fn test_set_active_model(&mut self, model_id: Option<ModelId>) {
        self.active_model = model_id;
    }
}
