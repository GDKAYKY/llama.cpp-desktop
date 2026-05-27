use std::sync::Arc;

use super::models::{GpuInfo, GpuUtilization, GpuVendor, ProcessGpuMetrics};
use super::traits::GpuMetricsProvider;

#[cfg(windows)]
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};

pub struct NvmlMetricsProvider {
    #[cfg(windows)]
    nvml: Arc<Nvml>,
}

impl NvmlMetricsProvider {
    #[cfg(windows)]
    pub fn new() -> Result<Self, String> {
        Nvml::init()
            .map(|nvml| Self { nvml: Arc::new(nvml) })
            .map_err(|e| e.to_string())
    }

    #[cfg(not(windows))]
    pub fn new() -> Result<Self, String> {
        Err("NVML is only available on Windows targets".to_string())
    }
}

#[cfg(windows)]
impl NvmlMetricsProvider {
    fn gather_for_device(&self, idx: u32, pid: u32) -> Option<ProcessGpuMetrics> {
        let device = self.nvml.device_by_index(idx).ok()?;
        let name = device.name().ok().unwrap_or_default();
        let mem_info = device.memory_info().ok()?;
        let total_mb = mem_info.total / 1024 / 1024;

        let mut vram_used_mb = None;

        if let Ok(processes) = device.running_compute_processes() {
            if let Some(proc_info) = processes.iter().find(|p| p.pid == pid) {
                vram_used_mb = match proc_info.used_gpu_memory {
                    nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => {
                        Some(bytes / 1024 / 1024)
                    }
                    nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => None,
                };
            }
        }

        if vram_used_mb.is_none() {
            if let Ok(processes) = device.running_graphics_processes() {
                if let Some(proc_info) = processes.iter().find(|p| p.pid == pid) {
                    vram_used_mb = match proc_info.used_gpu_memory {
                        nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => {
                            Some(bytes / 1024 / 1024)
                        }
                        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => None,
                    };
                }
            }
        }

        let utilization = match device.process_utilization_stats(None) {
            Ok(stats) => stats
                .into_iter()
                .find(|s| s.pid == pid)
                .map(|s| GpuUtilization::Precise {
                    sm: s.sm_util as f32,
                    mem: s.mem_util as f32,
                })
                .unwrap_or(GpuUtilization::Unavailable),
            Err(_) => GpuUtilization::Unavailable,
        };

        let temperature = device.temperature(TemperatureSensor::Gpu).ok();

        Some(ProcessGpuMetrics {
            gpu_index: idx,
            gpu_name: name,
            vram_used_mb: vram_used_mb.unwrap_or(0),
            vram_total_mb: total_mb,
            temperature,
            utilization,
        })
    }
}

#[cfg(windows)]
impl GpuMetricsProvider for NvmlMetricsProvider {
    fn metrics_for_pid(&self, pid: u32) -> Vec<ProcessGpuMetrics> {
        let device_count = match self.nvml.device_count() {
            Ok(count) => count,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::new();
        for idx in 0..device_count {
            if let Some(metrics) = self.gather_for_device(idx, pid) {
                if metrics.vram_used_mb > 0 || !matches!(metrics.utilization, GpuUtilization::Unavailable) {
                    result.push(metrics);
                }
            }
        }
        result
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        let count = match self.nvml.device_count() {
            Ok(count) => count,
            Err(_) => return Vec::new(),
        };

        (0..count)
            .filter_map(|idx| {
                let device = self.nvml.device_by_index(idx).ok()?;
                let mem = device.memory_info().ok()?;
                Some(GpuInfo {
                    index: idx,
                    name: device.name().ok().unwrap_or_default(),
                    vram_total_mb: mem.total / 1024 / 1024,
                    vendor: GpuVendor::Nvidia,
                })
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "nvml"
    }
}
