use super::models::{GpuInfo, ProcessGpuMetrics};
use super::traits::GpuMetricsProvider;

pub struct CompositeGpuMetricsProvider {
    providers: Vec<Box<dyn GpuMetricsProvider>>,
}

impl CompositeGpuMetricsProvider {
    pub fn new(providers: Vec<Box<dyn GpuMetricsProvider>>) -> Self {
        Self { providers }
    }
}

impl GpuMetricsProvider for CompositeGpuMetricsProvider {
    fn metrics_for_pid(&self, pid: u32) -> Vec<ProcessGpuMetrics> {
        for provider in &self.providers {
            let metrics = provider.metrics_for_pid(pid);
            if !metrics.is_empty() {
                return metrics;
            }
        }
        Vec::new()
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        for provider in &self.providers {
            let gpus = provider.available_gpus();
            if !gpus.is_empty() {
                return gpus;
            }
        }
        Vec::new()
    }

    fn name(&self) -> &'static str {
        "composite"
    }
}
