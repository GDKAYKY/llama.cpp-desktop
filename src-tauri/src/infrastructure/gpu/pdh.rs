use std::collections::HashMap;
use std::sync::Mutex;

use super::models::{GpuInfo, GpuUtilization, GpuVendor, ProcessGpuMetrics};
use super::traits::GpuMetricsProvider;

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
        let state = build_state()?;
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
                    / 1024 / 1024;
                let vram_total_mb = state
                    .adapter_totals
                    .get(&adapter_key)
                    .copied()
                    .or(metrics.dedicated_limit)
                    .unwrap_or(0)
                    / 1024 / 1024;

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
fn build_state() -> Result<PdhState, String> {
    unsafe {
        let mut query: isize = 0;
        if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != ERROR_SUCCESS.0 {
            return Err("failed to open PDH query".to_string());
        }

        let mut counters = Vec::new();
        let mut adapter_totals = HashMap::new();

        add_wildcard_counters(
            query,
            "\\GPU Engine(*)\\Utilization Percentage",
            CounterKind::EngineUtil,
            &mut counters,
            &mut adapter_totals,
        )?;
        add_wildcard_counters(
            query,
            "\\GPU Process Memory(*)\\Dedicated Usage",
            CounterKind::DedicatedUsage,
            &mut counters,
            &mut adapter_totals,
        )?;
        add_wildcard_counters(
            query,
            "\\GPU Process Memory(*)\\Shared Usage",
            CounterKind::SharedUsage,
            &mut counters,
            &mut adapter_totals,
        )?;
        add_wildcard_counters(
            query,
            "\\GPU Adapter Memory(*)\\Dedicated Limit",
            CounterKind::DedicatedLimit,
            &mut counters,
            &mut adapter_totals,
        )?;

        let _ = PdhCollectQueryData(query);

        for counter in counters.iter().filter(|c| matches!(c.kind, CounterKind::DedicatedLimit)) {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            let status = PdhGetFormattedCounterValue(counter.handle, PDH_FMT_DOUBLE, None, &mut value);
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
fn add_wildcard_counters(
    query: isize,
    wildcard: &str,
    kind: CounterKind,
    counters: &mut Vec<CounterEntry>,
    adapter_totals: &mut HashMap<String, u64>,
) -> Result<(), String> {
    let paths = expand_wildcard_paths(wildcard)?;

    for path in paths {
        let parsed = match kind {
            CounterKind::DedicatedLimit => parse_adapter_counter_key(&path).map(|adapter_key| (0, adapter_key)),
            _ => parse_instance_key(&path),
        };

        let Some((pid, adapter_key)) = parsed else {
            continue;
        };

        let mut counter: isize = 0;
        let wide = to_wide(&path);
        let status = unsafe { PdhAddEnglishCounterW(query, PCWSTR(wide.as_ptr()), 0, &mut counter) };
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
fn expand_wildcard_paths(wildcard: &str) -> Result<Vec<String>, String> {
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
