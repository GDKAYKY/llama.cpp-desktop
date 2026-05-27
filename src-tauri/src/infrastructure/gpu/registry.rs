use std::sync::Arc;

use super::composite::CompositeGpuMetricsProvider;
use super::models::{GpuInfo, GpuVendor, ProcessGpuMetrics};
use super::nvml::NvmlMetricsProvider;
use super::pdh::PdhMetricsProvider;
use super::traits::GpuMetricsProvider;

pub struct NullGpuMetricsProvider;

impl GpuMetricsProvider for NullGpuMetricsProvider {
    fn metrics_for_pid(&self, _pid: u32) -> Vec<ProcessGpuMetrics> {
        Vec::new()
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        Vec::new()
    }

    fn name(&self) -> &'static str {
        "null"
    }
}

fn nvml_provider() -> Option<Box<dyn GpuMetricsProvider>> {
    NvmlMetricsProvider::new().ok().map(|p| Box::new(p) as Box<dyn GpuMetricsProvider>)
}

fn pdh_provider() -> Option<Box<dyn GpuMetricsProvider>> {
    PdhMetricsProvider::new().ok().map(|p| Box::new(p) as Box<dyn GpuMetricsProvider>)
}

pub fn build_gpu_metrics_provider() -> Arc<dyn GpuMetricsProvider> {
    let mut providers: Vec<Box<dyn GpuMetricsProvider>> = Vec::new();

    if let Some(nvml) = nvml_provider() {
        providers.push(nvml);
    }

    if let Some(pdh) = pdh_provider() {
        providers.push(pdh);
    }

    if providers.is_empty() {
        Arc::new(NullGpuMetricsProvider)
    } else if providers.len() == 1 {
        Arc::from(providers.remove(0))
    } else {
        Arc::new(CompositeGpuMetricsProvider::new(providers))
    }
}

pub fn vendor_from_name(name: &str) -> GpuVendor {
    let lower = name.to_lowercase();
    if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") {
        GpuVendor::Nvidia
    } else if lower.contains("amd") || lower.contains("radeon") {
        GpuVendor::Amd
    } else if lower.contains("intel") {
        GpuVendor::Intel
    } else {
        GpuVendor::Unknown
    }
}
