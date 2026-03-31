use llama_desktop_lib::infrastructure::metrics::{MetricsProvider, SystemMetricsProvider};

#[test]
fn test_metrics_provider_nonexistent_pid() {
    let provider = SystemMetricsProvider::new();
    let metrics = provider.snapshot_for_pid(999999);
    assert!(metrics.is_none());
}

#[test]
fn test_metrics_provider_current_process() {
    let provider = SystemMetricsProvider::new();
    let current_pid = std::process::id();
    
    // Give system time to collect metrics
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let metrics = provider.snapshot_for_pid(current_pid);
    assert!(metrics.is_some());
    
    let m = metrics.unwrap();
    assert!(m.cpu_usage >= 0.0);
    assert!(m.mem_usage > 0);
}

#[test]
fn test_metrics_provider_poisoned_mutex() {
    let provider = SystemMetricsProvider::new();
    provider.test_poison_mutex();
    
    // Should return None when mutex is poisoned
    let metrics = provider.snapshot_for_pid(std::process::id());
    assert!(metrics.is_none());
}
