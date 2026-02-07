use std::process::Command;
use std::sync::Mutex;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct NvidiaSmi;

struct CommandOutput {
    success: bool,
    stdout: String,
}

impl NvidiaSmi {
    pub fn get_gpu_metrics_for_pid(pid: u32) -> (Option<f32>, Option<f32>) {
        let mut gpu_load = None;
        let mut vram_usage = None;

        // 1. Try to get system-wide GPU load and memory as a baseline
        if let Ok(output) = run_nvidia_smi(&[
            "--query-gpu=utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ]) {
            if output.success {
                let (parsed_gpu, parsed_vram) = parse_gpu_query(&output.stdout);
                gpu_load = parsed_gpu;
                vram_usage = parsed_vram;
            }
        }

        // 2. Try to refine with per-process memory if possible
        if let Ok(output) = run_nvidia_smi(&[
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ]) {
            if output.success {
                let _ = parse_process_list(&output.stdout, pid);
            }
        }

        (gpu_load, vram_usage)
    }
}

fn parse_gpu_query(stdout: &str) -> (Option<f32>, Option<f32>) {
    let mut gpu_load = None;
    let mut vram_usage = None;

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

    (gpu_load, vram_usage)
}

fn parse_process_list(stdout: &str, pid: u32) -> Option<u32> {
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            if let Ok(proc_pid) = parts[0].parse::<u32>() {
                if proc_pid == pid {
                    return Some(proc_pid);
                }
            }
        }
    }
    None
}

fn run_nvidia_smi(args: &[&str]) -> std::io::Result<CommandOutput> {
    if let Some(output) = take_stubbed_output() {
        return Ok(output);
    }
    let mut cmd = Command::new("nvidia-smi");
    #[cfg(windows)]
    {
        // Avoid flashing a console window when invoking nvidia-smi.
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.args(args).output()?;
    Ok(CommandOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

static NVIDIA_SMI_OUTPUTS: Mutex<Vec<CommandOutput>> = Mutex::new(Vec::new());

fn take_stubbed_output() -> Option<CommandOutput> {
    let mut outputs = NVIDIA_SMI_OUTPUTS.lock().expect("outputs lock");
    if outputs.is_empty() {
        return None;
    }
    Some(outputs.remove(0))
}

pub fn test_set_nvidia_smi_outputs(outputs: Vec<(bool, String)>) {
    let mut state = NVIDIA_SMI_OUTPUTS.lock().expect("outputs lock");
    *state = outputs
        .into_iter()
        .map(|(success, stdout)| CommandOutput { success, stdout })
        .collect();
}

pub fn test_parse_gpu_query(stdout: &str) -> (Option<f32>, Option<f32>) {
    parse_gpu_query(stdout)
}

pub fn test_parse_process_list(stdout: &str, pid: u32) -> Option<u32> {
    parse_process_list(stdout, pid)
}
