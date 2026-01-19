use crate::services::llama_cpp::LlamaCppService;

pub struct AppState {
    pub llama_service: LlamaCppService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            llama_service: LlamaCppService::new(),
        }
    }
}
