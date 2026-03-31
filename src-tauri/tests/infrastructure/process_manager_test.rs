
use llama_desktop_lib::infrastructure::llama::process::{LlamaProcessManager, ProcessManager};
use llama_desktop_lib::models::ModelId;
use std::sync::Arc;
use tokio::process::Command;

#[tokio::test]
async fn test_process_manager_register_and_get() {
    let manager = Arc::new(LlamaProcessManager::new());
    let model_id = ModelId("test:model:v1".to_string());
    
    // Create a dummy child process (sleep command)
    let child = Command::new("sleep").arg("1000").spawn().unwrap();
    let pid = child.id().unwrap();
    
    manager.register(model_id.clone(), child);
    
    let retrieved_pid = manager.get_pid(&model_id);
    assert_eq!(retrieved_pid, Some(pid));
    
    // Cleanup
    let mut child = manager.remove(&model_id).unwrap();
    let _ = child.kill().await;
}

#[tokio::test]
async fn test_process_manager_remove() {
    let manager = Arc::new(LlamaProcessManager::new());
    let model_id = ModelId("test:model:v1".to_string());
    
    let child = Command::new("sleep").arg("1000").spawn().unwrap();
    manager.register(model_id.clone(), child);
    
    let removed = manager.remove(&model_id);
    assert!(removed.is_some());
    
    let pid = manager.get_pid(&model_id);
    assert_eq!(pid, None);
    
    // Cleanup
    let mut child = removed.unwrap();
    let _ = child.kill().await;
}

#[tokio::test]
async fn test_process_manager_multiple_models() {
    let manager = Arc::new(LlamaProcessManager::new());
    let model1 = ModelId("test:model1:v1".to_string());
    let model2 = ModelId("test:model2:v1".to_string());
    
    let child1 = Command::new("sleep").arg("1000").spawn().unwrap();
    let child2 = Command::new("sleep").arg("1000").spawn().unwrap();
    let pid1 = child1.id().unwrap();
    let pid2 = child2.id().unwrap();
    
    manager.register(model1.clone(), child1);
    manager.register(model2.clone(), child2);
    
    assert_eq!(manager.get_pid(&model1), Some(pid1));
    assert_eq!(manager.get_pid(&model2), Some(pid2));
    
    // Cleanup
    let mut c1 = manager.remove(&model1).unwrap();
    let mut c2 = manager.remove(&model2).unwrap();
    let _ = c1.kill().await;
    let _ = c2.kill().await;
}

#[test]
fn test_process_manager_get_nonexistent() {
    let manager = Arc::new(LlamaProcessManager::new());
    let model_id = ModelId("nonexistent:model:v1".to_string());
    
    let pid = manager.get_pid(&model_id);
    assert_eq!(pid, None);
}

#[test]
fn test_process_manager_poisoned_mutex() {
    let manager = Arc::new(LlamaProcessManager::new());
    manager.test_poison_mutex();
    
    let model_id = ModelId("test:model:v1".to_string());
    let pid = manager.get_pid(&model_id);
    assert_eq!(pid, None);
}
