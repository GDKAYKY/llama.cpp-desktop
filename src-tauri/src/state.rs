use crate::services::llama::LlamaCppService;
use crate::services::orchestrator::ChatOrchestrator;

pub struct AppState {
    pub llama_service: LlamaCppService,
    pub orchestrator: ChatOrchestrator,
}

impl AppState {
    pub fn new(models_path: std::path::PathBuf) -> Self {
        let llama_service = LlamaCppService::new(models_path);
        let orchestrator = ChatOrchestrator::new(llama_service.clone());
        Self {
            llama_service,
            orchestrator,
        }
    }
}
