use std::collections::HashMap;
use std::sync::Mutex;

use tokio::process::Child;

use crate::models::ModelId;

pub trait ProcessManager: Send + Sync {
    fn register(&self, model_id: ModelId, child: Child);
    fn get_pid(&self, model_id: &ModelId) -> Option<u32>;
    fn remove(&self, model_id: &ModelId) -> Option<Child>;
}

pub struct LlamaProcessManager {
    children: Mutex<HashMap<ModelId, Child>>,
}

impl LlamaProcessManager {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(HashMap::new()),
        }
    }
}

impl ProcessManager for LlamaProcessManager {
    fn register(&self, model_id: ModelId, child: Child) {
        if let Ok(mut children) = self.children.lock() {
            children.insert(model_id, child);
        }
    }

    fn get_pid(&self, model_id: &ModelId) -> Option<u32> {
        let children = self.children.lock().ok()?;
        children.get(model_id).and_then(|child| child.id())
    }

    fn remove(&self, model_id: &ModelId) -> Option<Child> {
        let mut children = self.children.lock().ok()?;
        children.remove(model_id)
    }
}
