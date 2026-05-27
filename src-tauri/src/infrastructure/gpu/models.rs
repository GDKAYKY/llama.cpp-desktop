use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub index: u32,
    pub name: String,
    pub vram_total_mb: u64,
    pub vendor: GpuVendor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuUtilization {
    Precise { sm: f32, mem: f32 },
    Engine { gpu: f32 },
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessGpuMetrics {
    pub gpu_index: u32,
    pub gpu_name: String,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub temperature: Option<u32>,
    pub utilization: GpuUtilization,
}
