use llama_desktop_lib::infrastructure::metrics::{MetricsProvider, SystemMetricsProvider};

#[test]
fn snapshot_for_current_pid_returns_metrics() {
    let provider = SystemMetricsProvider::new();
    let pid = std::process::id();
    let metrics = provider.snapshot_for_pid(pid);
    assert!(metrics.is_some());
    let metrics = metrics.expect("metrics");
    assert!(metrics.cpu_usage >= 0.0);
}

#[test]
fn snapshot_returns_none_for_unknown_pid() {
    let provider = SystemMetricsProvider::new();
    let metrics = provider.snapshot_for_pid(u32::MAX);
    assert!(metrics.is_none());
}

#[test]
fn snapshot_returns_none_when_mutex_poisoned() {
    let provider = SystemMetricsProvider::new();
    provider.test_poison_mutex();
    let pid = std::process::id();
    let metrics = provider.snapshot_for_pid(pid);
    assert!(metrics.is_none());
}
