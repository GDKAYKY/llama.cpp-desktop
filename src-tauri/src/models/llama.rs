use serde::{Deserialize, Serialize};
use tokio::process::Child;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    pub llama_cpp_path: String,
    pub model_path: String,
    pub port: u16,
    pub ctx_size: u32,
    pub parallel: u32,
    pub n_gpu_layers: i32,
}

pub enum ModelState {
    Stopped,
    Starting,
    Running {
        port: u16,
        child: Child,
        config: LlamaCppConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub cpu_usage: f32,
    pub mem_usage: u64,
    pub gpu_usage: Option<f32>,
    pub vram_usage: Option<f32>, // VRAM usage percentage (0-100%)
}
