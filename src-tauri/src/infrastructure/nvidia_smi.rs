use std::process::Command;

pub struct NvidiaSmi;

impl NvidiaSmi {
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
                                // Specific process found, but we use system-wide for now as requested
                            }
                        }
                    }
                }
            }
        }

        (gpu_load, vram_usage)
    }
}
