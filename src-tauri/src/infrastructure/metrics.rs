use std::sync::{Arc, Mutex};
use std::panic::AssertUnwindSafe;

use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

use crate::infrastructure::gpu::{build_gpu_metrics_provider, GpuMetricsProvider, GpuUtilization, ProcessGpuMetrics};
use crate::models::ServerMetrics;

pub trait MetricsProvider: Send + Sync {
    fn snapshot_for_pid(&self, pid: u32) -> Option<ServerMetrics>;
}

pub struct SystemMetricsProvider {
    sys: Mutex<System>,
    gpu: Arc<dyn GpuMetricsProvider>,
}

impl SystemMetricsProvider {
    pub fn new() -> Self {
        Self {
            gpu: build_gpu_metrics_provider(),
            sys: Mutex::new(System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(ProcessRefreshKind::nothing().with_cpu().with_memory())
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
            )),
        }
    }
}

impl MetricsProvider for SystemMetricsProvider {
    fn snapshot_for_pid(&self, pid: u32) -> Option<ServerMetrics> {
        let mut sys = self.sys.lock().ok()?;
        let sys_pid = sysinfo::Pid::from(pid as usize);

        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[sys_pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        let process = sys.process(sys_pid)?;
        let gpu_metrics = self.gpu.metrics_for_pid(pid);
        let (gpu_usage, vram_usage) = aggregate_gpu_metrics(&gpu_metrics);

        Some(ServerMetrics {
            cpu_usage: process.cpu_usage(),
            mem_usage: process.memory(),
            gpu_usage,
            vram_usage,
            gpu_instances: gpu_metrics,
        })
    }
}

impl SystemMetricsProvider {
    pub fn test_poison_mutex(&self) {
        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = self.sys.lock().unwrap();
            panic!("poison");
        }));
    }
}

fn aggregate_gpu_metrics(metrics: &[ProcessGpuMetrics]) -> (Option<f32>, Option<f32>) {
    if metrics.is_empty() {
        return (None, None);
    }

    let mut gpu_usage: Option<f32> = None;
    let mut used_total = 0u64;
    let mut total_total = 0u64;

    for metric in metrics {
        match metric.utilization {
            GpuUtilization::Precise { sm, .. } => {
                gpu_usage = Some(match gpu_usage {
                    Some(current) => current.max(sm),
                    None => sm,
                });
            }
            GpuUtilization::Engine { gpu } => {
                gpu_usage = Some(match gpu_usage {
                    Some(current) => current.max(gpu),
                    None => gpu,
                });
            }
            GpuUtilization::Unavailable => {}
        }

        used_total = used_total.saturating_add(metric.vram_used_mb);
        total_total = total_total.saturating_add(metric.vram_total_mb);
    }

    let vram_usage = if total_total > 0 {
        Some((used_total as f32 / total_total as f32) * 100.0)
    } else {
        None
    };

    (gpu_usage, vram_usage)
}
