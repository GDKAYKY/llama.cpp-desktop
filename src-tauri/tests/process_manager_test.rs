use llama_desktop_lib::infrastructure::llama::process::{LlamaProcessManager, ProcessManager};
use llama_desktop_lib::models::ModelId;

async fn spawn_sleep_child() -> tokio::process::Child {
    #[cfg(windows)]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(["/C", "ping", "127.0.0.1", "-n", "6"]);
        cmd
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("sleep");
        cmd.arg("5");
        cmd
    };
    cmd.spawn().expect("spawn child")
}

#[tokio::test]
async fn register_get_pid_and_remove_child() {
    let manager = LlamaProcessManager::new();
    let model_id = ModelId("model-1".to_string());
    let child = spawn_sleep_child().await;
    let pid = child.id().expect("pid");
    manager.register(model_id.clone(), child);

    assert_eq!(manager.get_pid(&model_id), Some(pid));

    let mut removed = manager.remove(&model_id).expect("removed child");
    let _ = removed.kill().await;
    let _ = removed.wait().await;
    assert!(manager.get_pid(&model_id).is_none());
}

#[tokio::test]
async fn handles_poisoned_mutex_gracefully() {
    let manager = LlamaProcessManager::new();
    manager.test_poison_mutex();

    let model_id = ModelId("model-2".to_string());
    let child = spawn_sleep_child().await;
    manager.register(model_id.clone(), child);
    assert!(manager.get_pid(&model_id).is_none());
    assert!(manager.remove(&model_id).is_none());
}
