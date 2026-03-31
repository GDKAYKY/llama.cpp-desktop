use llama_desktop_lib::infrastructure::nvidia_smi::{
    test_parse_gpu_query, test_parse_process_list, test_set_nvidia_smi_outputs, NvidiaSmi,
};

#[test]
fn test_parse_gpu_query_valid() {
    let stdout = "45, 2048, 8192\n";
    let (gpu, vram) = test_parse_gpu_query(stdout);

    assert_eq!(gpu, Some(45.0));
    assert_eq!(vram, Some(25.0)); // 2048/8192 * 100
}

#[test]
fn test_parse_gpu_query_multiple_gpus() {
    let stdout = "60, 4096, 12288\n30, 1024, 8192\n";
    let (gpu, vram) = test_parse_gpu_query(stdout);

    // Should parse first line
    assert_eq!(gpu, Some(60.0));
    assert!(vram.is_some());
}

#[test]
fn test_parse_gpu_query_invalid() {
    let stdout = "invalid, data, here\n";
    let (gpu, vram) = test_parse_gpu_query(stdout);

    assert_eq!(gpu, None);
    assert_eq!(vram, None);
}

#[test]
fn test_parse_gpu_query_empty() {
    let stdout = "";
    let (gpu, vram) = test_parse_gpu_query(stdout);

    assert_eq!(gpu, None);
    assert_eq!(vram, None);
}

#[test]
fn test_parse_gpu_query_zero_total_vram() {
    let stdout = "50, 2048, 0\n";
    let (gpu, vram) = test_parse_gpu_query(stdout);

    assert_eq!(gpu, Some(50.0));
    assert_eq!(vram, None); // Division by zero protection
}

#[test]
fn test_parse_process_list_found() {
    let stdout = "1234, 512\n5678, 1024\n9999, 256\n";
    let result = test_parse_process_list(stdout, 5678);

    assert_eq!(result, Some(5678));
}

#[test]
fn test_parse_process_list_not_found() {
    let stdout = "1234, 512\n5678, 1024\n";
    let result = test_parse_process_list(stdout, 9999);

    assert_eq!(result, None);
}

#[test]
fn test_parse_process_list_empty() {
    let stdout = "";
    let result = test_parse_process_list(stdout, 1234);

    assert_eq!(result, None);
}

#[test]
fn test_parse_process_list_invalid_format() {
    let stdout = "invalid, data\n1234\n";
    let result = test_parse_process_list(stdout, 1234);

    assert_eq!(result, None);
}

#[test]
fn test_get_gpu_metrics_with_stub_success() {
    test_set_nvidia_smi_outputs(vec![
        (true, "75, 6144, 8192\n".to_string()),
        (true, "12345, 2048\n".to_string()),
    ]);

    let (gpu, vram) = NvidiaSmi::get_gpu_metrics_for_pid(12345);

    assert_eq!(gpu, Some(75.0));
    assert_eq!(vram, Some(75.0)); // 6144/8192 * 100
}

#[test]
fn test_get_gpu_metrics_with_stub_failure() {
    test_set_nvidia_smi_outputs(vec![(false, "".to_string())]);

    let (gpu, vram) = NvidiaSmi::get_gpu_metrics_for_pid(12345);

    assert_eq!(gpu, None);
    assert_eq!(vram, None);
}
