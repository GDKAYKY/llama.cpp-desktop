#[cfg(not(test))]
#[tauri::command]
pub fn get_available_gpus() -> Result<Vec<crate::infrastructure::gpu::GpuInfo>, String> {
    let provider = crate::infrastructure::gpu::build_gpu_metrics_provider();
    Ok(provider.available_gpus())
}
