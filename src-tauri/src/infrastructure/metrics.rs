use std::sync::Mutex;

use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

use crate::infrastructure::nvidia_smi::NvidiaSmi;
use crate::models::ServerMetrics;

pub trait MetricsProvider: Send + Sync {
    fn snapshot_for_pid(&self, pid: u32) -> Option<ServerMetrics>;
}

pub struct SystemMetricsProvider {
    sys: Mutex<System>,
}

impl SystemMetricsProvider {
    pub fn new() -> Self {
        Self {
            sys: Mutex::new(System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(ProcessRefreshKind::nothing().with_cpu())
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
            ProcessRefreshKind::nothing().with_cpu(),
        );

        let process = sys.process(sys_pid)?;
        let (gpu_usage, vram_usage) = NvidiaSmi::get_gpu_metrics_for_pid(pid);

        Some(ServerMetrics {
            cpu_usage: process.cpu_usage(),
            mem_usage: process.memory(),
            gpu_usage,
            vram_usage,
        })
    }
}

impl SystemMetricsProvider {
    pub fn test_poison_mutex(&self) {
        let _ = std::panic::catch_unwind(|| {
            let _guard = self.sys.lock().unwrap();
            panic!("poison");
        });
    }
}
