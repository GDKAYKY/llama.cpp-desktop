use super::models::{GpuInfo, ProcessGpuMetrics};

pub trait GpuMetricsProvider: Send + Sync {
    fn metrics_for_pid(&self, pid: u32) -> Vec<ProcessGpuMetrics>;
    fn available_gpus(&self) -> Vec<GpuInfo>;
    fn name(&self) -> &'static str;
}
