use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};
#[cfg(windows)]
use windows::core::{PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::ERROR_SUCCESS;
#[cfg(windows)]
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhExpandWildCardPathW,
    PdhGetFormattedCounterValue, PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
    PDH_MORE_DATA,
};
use wgpu::{Backends, Instance, InstanceDescriptor};

use crate::models::ServerMetrics;
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub trait GpuMetricsProvider: Send + Sync {
    fn metrics_for_pid(&self, pid: u32) -> Vec<ProcessGpuMetrics>;
    fn available_gpus(&self) -> Vec<GpuInfo>;
    fn name(&self) -> &'static str;
}

pub trait MetricsProvider: Send + Sync {
    fn snapshot_for_pid(&self, pid: u32) -> Option<ServerMetrics>;
}

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

pub fn build_gpu_metrics_provider() -> Arc<dyn GpuMetricsProvider> {
    let mut providers: Vec<Box<dyn GpuMetricsProvider>> = Vec::new();

    if let Some(nvml) = NvmlMetricsProvider::new().ok() {
        providers.push(Box::new(nvml));
    }

    if let Some(pdh) = PdhMetricsProvider::new().ok() {
        providers.push(Box::new(pdh));
    }

    if let Some(wgpu) = WgpuDetector::new().ok() {
        providers.push(Box::new(wgpu));
    }

    if providers.is_empty() {
        Arc::new(NullGpuMetricsProvider)
    } else if providers.len() == 1 {
        Arc::from(providers.remove(0))
    } else {
        Arc::new(FallbackGpuMetricsProvider::new(providers))
    }
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

    pub fn test_poison_mutex(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = self.sys.lock().unwrap();
            panic!("poison");
        }));
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

pub struct FallbackGpuMetricsProvider {
    providers: Vec<Box<dyn GpuMetricsProvider>>,
}

impl FallbackGpuMetricsProvider {
    pub fn new(providers: Vec<Box<dyn GpuMetricsProvider>>) -> Self {
        Self { providers }
    }
}

impl GpuMetricsProvider for FallbackGpuMetricsProvider {
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
        let mut all_gpus = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for provider in &self.providers {
            for gpu in provider.available_gpus() {
                if seen_names.insert(gpu.name.clone()) {
                    all_gpus.push(gpu);
                }
            }
        }

        all_gpus
            .into_iter()
            .enumerate()
            .map(|(index, mut gpu)| {
                gpu.index = index as u32;
                gpu
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "fallback"
    }
}

pub struct WgpuDetector {
    gpus: Vec<GpuInfo>,
}

impl WgpuDetector {
    pub fn new() -> Result<Self, String> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapters = instance.enumerate_adapters(Backends::all());
        let mut gpus = Vec::new();

        for (index, adapter) in adapters.into_iter().enumerate() {
            let info = adapter.get_info();

            if matches!(
                info.device_type,
                wgpu::DeviceType::DiscreteGpu
                    | wgpu::DeviceType::IntegratedGpu
                    | wgpu::DeviceType::VirtualGpu
            ) {
                gpus.push(GpuInfo {
                    index: index as u32,
                    name: info.name.clone(),
                    vram_total_mb: 0,
                    vendor: vendor_from_name(&info.name),
                });
            }
        }

        if gpus.is_empty() {
            return Err("No GPUs found via wgpu".to_string());
        }

        Ok(Self { gpus })
    }
}

impl GpuMetricsProvider for WgpuDetector {
    fn metrics_for_pid(&self, _pid: u32) -> Vec<ProcessGpuMetrics> {
        Vec::new()
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        self.gpus.clone()
    }

    fn name(&self) -> &'static str {
        "wgpu_detector"
    }
}

// ---------------------------------------------------------------------------
// NVIDIA NVML metrics provider
// ---------------------------------------------------------------------------

pub struct NvmlMetricsProvider {
    #[cfg(windows)]
    nvml: Arc<Nvml>,
}

impl NvmlMetricsProvider {
    #[cfg(windows)]
    pub fn new() -> Result<Self, String> {
        Nvml::init()
            .map(|nvml| Self {
                nvml: Arc::new(nvml),
            })
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
                if metrics.vram_used_mb > 0
                    || !matches!(metrics.utilization, GpuUtilization::Unavailable)
                {
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

#[cfg(not(windows))]
impl GpuMetricsProvider for NvmlMetricsProvider {
    fn metrics_for_pid(&self, _pid: u32) -> Vec<ProcessGpuMetrics> {
        Vec::new()
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        Vec::new()
    }

    fn name(&self) -> &'static str {
        "nvml"
    }
}

// ---------------------------------------------------------------------------
// Windows PDH (Performance Data Helper) metrics provider
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CounterKind {
    EngineUtil,
    DedicatedUsage,
    SharedUsage,
    DedicatedLimit,
}

#[derive(Clone, Debug)]
struct CounterEntry {
    pid: u32,
    adapter_key: String,
    kind: CounterKind,
    #[cfg(windows)]
    handle: isize,
}

#[cfg(windows)]
struct PdhState {
    query: isize,
    counters: Vec<CounterEntry>,
    adapter_totals: HashMap<String, u64>,
}

#[cfg(windows)]
impl Drop for PdhState {
    fn drop(&mut self) {
        unsafe {
            let _ = PdhCloseQuery(self.query);
        }
    }
}

pub struct PdhMetricsProvider {
    #[cfg(windows)]
    state: Mutex<Option<PdhState>>,
}

impl PdhMetricsProvider {
    #[cfg(windows)]
    pub fn new() -> Result<Self, String> {
        let state = pdh_build_state()?;
        Ok(Self {
            state: Mutex::new(Some(state)),
        })
    }

    #[cfg(not(windows))]
    pub fn new() -> Result<Self, String> {
        Err("PDH is only available on Windows targets".to_string())
    }
}

#[cfg(windows)]
impl PdhMetricsProvider {
    fn metrics_for_pid_inner(state: &mut PdhState, pid: u32) -> Vec<ProcessGpuMetrics> {
        unsafe {
            let _ = PdhCollectQueryData(state.query);
        }

        let mut by_adapter: HashMap<String, PartialMetrics> = HashMap::new();

        for counter in &state.counters {
            if counter.pid != pid {
                continue;
            }

            let mut value = PDH_FMT_COUNTERVALUE::default();
            let status = unsafe {
                PdhGetFormattedCounterValue(counter.handle, PDH_FMT_DOUBLE, None, &mut value)
            };
            if status != ERROR_SUCCESS.0 {
                continue;
            }

            let raw = unsafe { value.Anonymous.doubleValue as f32 };
            let entry = by_adapter
                .entry(counter.adapter_key.clone())
                .or_insert_with(PartialMetrics::default);

            match counter.kind {
                CounterKind::EngineUtil => {
                    entry.gpu_util = Some(match entry.gpu_util {
                        Some(current) => current.max(raw),
                        None => raw,
                    });
                }
                CounterKind::DedicatedUsage => {
                    entry.dedicated_usage = Some(raw.max(0.0) as u64);
                }
                CounterKind::SharedUsage => {
                    entry.shared_usage = Some(raw.max(0.0) as u64);
                }
                CounterKind::DedicatedLimit => {
                    entry.dedicated_limit = Some(raw.max(0.0) as u64);
                }
            }
        }

        let mut adapters: Vec<_> = by_adapter.into_iter().collect();
        adapters.sort_by(|a, b| a.0.cmp(&b.0));

        adapters
            .into_iter()
            .enumerate()
            .map(|(index, (adapter_key, metrics))| {
                let vram_used_mb = metrics
                    .dedicated_usage
                    .unwrap_or(0)
                    .saturating_add(metrics.shared_usage.unwrap_or(0))
                    / 1024
                    / 1024;
                let vram_total_mb = state
                    .adapter_totals
                    .get(&adapter_key)
                    .copied()
                    .or(metrics.dedicated_limit)
                    .unwrap_or(0)
                    / 1024
                    / 1024;

                ProcessGpuMetrics {
                    gpu_index: index as u32,
                    gpu_name: adapter_key,
                    vram_used_mb,
                    vram_total_mb,
                    temperature: None,
                    utilization: metrics
                        .gpu_util
                        .map(|gpu| GpuUtilization::Engine { gpu })
                        .unwrap_or(GpuUtilization::Unavailable),
                }
            })
            .collect()
    }
}

#[cfg(windows)]
impl GpuMetricsProvider for PdhMetricsProvider {
    fn metrics_for_pid(&self, pid: u32) -> Vec<ProcessGpuMetrics> {
        let mut guard = match self.state.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        let state = match guard.as_mut() {
            Some(state) => state,
            None => return Vec::new(),
        };

        Self::metrics_for_pid_inner(state, pid)
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        let guard = match self.state.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        let state = match guard.as_ref() {
            Some(state) => state,
            None => return Vec::new(),
        };

        let mut adapters: Vec<_> = state.adapter_totals.iter().collect();
        adapters.sort_by(|a, b| a.0.cmp(b.0));

        let mut gpus = Vec::new();
        for (index, (adapter_key, total)) in adapters.into_iter().enumerate() {
            gpus.push(GpuInfo {
                index: index as u32,
                name: adapter_key.clone(),
                vram_total_mb: *total / 1024 / 1024,
                vendor: GpuVendor::Unknown,
            });
        }

        gpus
    }

    fn name(&self) -> &'static str {
        "pdh"
    }
}

#[cfg(not(windows))]
impl GpuMetricsProvider for PdhMetricsProvider {
    fn metrics_for_pid(&self, _pid: u32) -> Vec<ProcessGpuMetrics> {
        Vec::new()
    }

    fn available_gpus(&self) -> Vec<GpuInfo> {
        Vec::new()
    }

    fn name(&self) -> &'static str {
        "pdh"
    }
}

#[cfg(windows)]
#[derive(Default)]
struct PartialMetrics {
    gpu_util: Option<f32>,
    dedicated_usage: Option<u64>,
    shared_usage: Option<u64>,
    dedicated_limit: Option<u64>,
}

#[cfg(windows)]
fn pdh_build_state() -> Result<PdhState, String> {
    unsafe {
        let mut query: isize = 0;
        if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != ERROR_SUCCESS.0 {
            return Err("failed to open PDH query".to_string());
        }

        let mut counters = Vec::new();
        let mut adapter_totals = HashMap::new();

        pdh_add_wildcard_counters(
            query,
            "\\GPU Engine(*)\\Utilization Percentage",
            CounterKind::EngineUtil,
            &mut counters,
            &mut adapter_totals,
        )?;
        pdh_add_wildcard_counters(
            query,
            "\\GPU Process Memory(*)\\Dedicated Usage",
            CounterKind::DedicatedUsage,
            &mut counters,
            &mut adapter_totals,
        )?;
        pdh_add_wildcard_counters(
            query,
            "\\GPU Process Memory(*)\\Shared Usage",
            CounterKind::SharedUsage,
            &mut counters,
            &mut adapter_totals,
        )?;
        pdh_add_wildcard_counters(
            query,
            "\\GPU Adapter Memory(*)\\Dedicated Limit",
            CounterKind::DedicatedLimit,
            &mut counters,
            &mut adapter_totals,
        )?;

        let _ = PdhCollectQueryData(query);

        for counter in counters
            .iter()
            .filter(|c| matches!(c.kind, CounterKind::DedicatedLimit))
        {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            let status = PdhGetFormattedCounterValue(
                counter.handle,
                PDH_FMT_DOUBLE,
                None,
                &mut value,
            );
            if status == ERROR_SUCCESS.0 {
                let raw = value.Anonymous.doubleValue;
                adapter_totals.insert(counter.adapter_key.clone(), raw.max(0.0) as u64);
            }
        }

        Ok(PdhState {
            query,
            counters,
            adapter_totals,
        })
    }
}

#[cfg(windows)]
fn pdh_add_wildcard_counters(
    query: isize,
    wildcard: &str,
    kind: CounterKind,
    counters: &mut Vec<CounterEntry>,
    adapter_totals: &mut HashMap<String, u64>,
) -> Result<(), String> {
    let paths = pdh_expand_wildcard_paths(wildcard)?;

    for path in paths {
        let parsed = match kind {
            CounterKind::DedicatedLimit => {
                parse_adapter_counter_key(&path).map(|adapter_key| (0, adapter_key))
            }
            _ => parse_instance_key(&path),
        };

        let Some((pid, adapter_key)) = parsed else {
            continue;
        };

        let mut counter: isize = 0;
        let wide = to_wide(&path);
        let status =
            unsafe { PdhAddEnglishCounterW(query, PCWSTR(wide.as_ptr()), 0, &mut counter) };
        if status != ERROR_SUCCESS.0 {
            continue;
        }

        if matches!(kind, CounterKind::DedicatedLimit) {
            adapter_totals.entry(adapter_key.clone()).or_insert(0);
        }

        counters.push(CounterEntry {
            pid,
            adapter_key,
            kind,
            handle: counter,
        });
    }

    Ok(())
}

#[cfg(windows)]
fn pdh_expand_wildcard_paths(wildcard: &str) -> Result<Vec<String>, String> {
    unsafe {
        let wide = to_wide(wildcard);
        let mut len: u32 = 0;
        let status = PdhExpandWildCardPathW(
            PCWSTR::null(),
            PCWSTR(wide.as_ptr()),
            PWSTR::null(),
            &mut len,
            0,
        );

        if status != PDH_MORE_DATA {
            return Err(format!("failed to size wildcard expansion: {status}"));
        }

        let mut buffer = vec![0u16; len as usize];
        let status = PdhExpandWildCardPathW(
            PCWSTR::null(),
            PCWSTR(wide.as_ptr()),
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
            0,
        );

        if status != ERROR_SUCCESS.0 {
            return Err(format!("failed to expand wildcard: {status}"));
        }

        Ok(split_multi_sz(&buffer))
    }
}

#[cfg(windows)]
fn split_multi_sz(buffer: &[u16]) -> Vec<String> {
    let mut items = Vec::new();
    let mut start = 0usize;

    for (idx, ch) in buffer.iter().enumerate() {
        if *ch == 0 {
            if start == idx {
                break;
            }
            items.push(String::from_utf16_lossy(&buffer[start..idx]));
            start = idx + 1;
        }
    }

    items.retain(|s| !s.trim().is_empty());
    items
}

#[cfg(windows)]
fn parse_instance_key(path: &str) -> Option<(u32, String)> {
    let open = path.find('(')?;
    let close = path[open + 1..].find(')')? + open + 1;
    let instance = &path[open + 1..close];
    let pid = parse_pid(instance)?;
    let adapter_key = parse_adapter_key(instance).unwrap_or_else(|| instance.to_string());
    Some((pid, adapter_key))
}

#[cfg(windows)]
fn parse_pid(instance: &str) -> Option<u32> {
    let pid_pos = instance.find("pid_")? + 4;
    let tail = &instance[pid_pos..];
    let end = tail.find('_').unwrap_or(tail.len());
    tail[..end].parse().ok()
}

#[cfg(windows)]
fn parse_adapter_key(instance: &str) -> Option<String> {
    let luid_pos = instance.find("luid_")?;
    let tail = &instance[luid_pos + 5..];
    let end = tail
        .find("_engtype_")
        .or_else(|| tail.find("_phys_"))
        .unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

#[cfg(windows)]
fn parse_adapter_counter_key(path: &str) -> Option<String> {
    let open = path.find('(')?;
    let close = path[open + 1..].find(')')? + open + 1;
    let instance = &path[open + 1..close];
    let adapter_key = parse_adapter_key(instance).unwrap_or_else(|| instance.to_string());
    Some(adapter_key)
}

#[cfg(windows)]
fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
