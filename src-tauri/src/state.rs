use crate::services::llama::LlamaCppService;
use crate::services::mcp::McpService;
use crate::services::orchestrator::ChatOrchestrator;

pub struct AppState {
    pub llama_service: LlamaCppService,
    pub mcp_service: McpService,
    pub orchestrator: ChatOrchestrator,
}

impl AppState {
    pub fn new(models_path: std::path::PathBuf, mcp_config: crate::models::McpConfig) -> Self {
        let llama_service = LlamaCppService::new(models_path);
        let mcp_service = McpService::new(mcp_config);
        let orchestrator = ChatOrchestrator::new(llama_service.clone(), mcp_service.clone());
        Self {
            llama_service,
            mcp_service,
            orchestrator,
        }
    }
}
