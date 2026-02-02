use serde::{Deserialize, Serialize};
use std::process::Command;
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

impl ServerMetrics {
    pub fn get_gpu_metrics_for_pid(pid: u32) -> (Option<f32>, Option<f32>) {
        let mut gpu_load = None;
        let mut vram_usage = None;

        // 1. Try to get system-wide GPU load and memory as a baseline
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=utilization.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 3 {
                        gpu_load = parts[0].parse::<f32>().ok();
                        let used_vram = parts[1].parse::<f32>().ok();
                        let total_vram = parts[2].parse::<f32>().ok();

                        if let (Some(used), Some(total)) = (used_vram, total_vram) {
                            if total > 0.0 {
                                vram_usage = Some((used / total) * 100.0);
                            }
                        }
                    }
                }
            }
        }

        // 2. Try to refine with per-process memory if possible
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&[
                "--query-compute-apps=pid,used_memory",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        if let Ok(proc_pid) = parts[0].parse::<u32>() {
                            if proc_pid == pid {
                                // If we found our process, we can use its specific memory usage
                                // to calculate a more accurate VRAM % for the app itself if we wanted,
                                // but usually, the user wants to see the total system impact.
                                // For now, we'll stick to system-wide if it's already found,
                                // but we could parse parts[1] here.
                            }
                        }
                    }
                }
            }
        }

        (gpu_load, vram_usage)
    }
}
