use llama_desktop_lib::infrastructure::nvidia_smi::{
    test_parse_gpu_query, test_parse_process_list, test_set_nvidia_smi_outputs, NvidiaSmi,
};

#[test]
fn parse_gpu_query_handles_valid_line() {
    let (gpu, vram) = test_parse_gpu_query("10, 200, 400\n");
    assert_eq!(gpu, Some(10.0));
    assert_eq!(vram, Some(50.0));
}

#[test]
fn parse_gpu_query_handles_zero_total() {
    let (_gpu, vram) = test_parse_gpu_query("10, 0, 0\n");
    assert!(vram.is_none());
}

#[test]
fn parse_gpu_query_handles_invalid_line() {
    let (gpu, vram) = test_parse_gpu_query("bad line\n");
    assert!(gpu.is_none());
    assert!(vram.is_none());
}

#[test]
fn parse_process_list_finds_pid() {
    let found = test_parse_process_list("1234, 100\n5678, 200\n", 5678);
    assert_eq!(found, Some(5678));
}

#[test]
fn parse_process_list_handles_missing() {
    let found = test_parse_process_list("1234, 100\n", 5678);
    assert!(found.is_none());
}

#[test]
fn get_gpu_metrics_uses_stubbed_outputs() {
    test_set_nvidia_smi_outputs(vec![
        (true, "55, 300, 600\n".to_string()),
        (true, "999, 200\n".to_string()),
    ]);

    let (gpu, vram) = NvidiaSmi::get_gpu_metrics_for_pid(999);
    assert_eq!(gpu, Some(55.0));
    assert_eq!(vram, Some(50.0));
}

#[test]
fn get_gpu_metrics_handles_unsuccessful_commands() {
    test_set_nvidia_smi_outputs(vec![
        (false, "bad".to_string()),
        (false, "bad".to_string()),
    ]);

    let (gpu, vram) = NvidiaSmi::get_gpu_metrics_for_pid(123);
    assert!(gpu.is_none());
    assert!(vram.is_none());
}
