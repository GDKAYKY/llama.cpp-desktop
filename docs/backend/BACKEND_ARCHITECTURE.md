# Backend Architecture Standards

This document defines the mandatory code organization standards for the Rust backend in the Llama.cpp Desktop project.

## 1. Centralized Data Models

**Rule:** All shared data structures, including Structs, Enums, and Constants that represent data entities, must reside in `src-tauri/src/models/`.

### Directory Layout

The models are organized in `src-tauri/src/lib.rs` as:

```rust
pub mod models {
    pub mod app_settings_model;
    pub mod chat_model;
    pub mod llama_model;
    pub mod manifest_model;
    pub mod mcp_model;

    pub use app_settings_model::*;
    pub use chat_model::*;
    pub use llama_model::*;
    pub use manifest_model::*;
    pub use mcp_model::*;
}
```

### Model Files

- **`app_settings_model.rs`**: Global application settings
  - `AppConfig` - Application configuration (models directory, settings)
  - `AppSettings` - User preferences
  
- **`chat_model.rs`**: Chat API contracts and state
  - `ChatMessage` - Individual chat message
  - `ChatRequest` - Request to send message
  - `ChatResponse` - Streaming response
  - `ToolCall` - MCP tool invocation
  - `ToolResult` - Tool execution result
  
- **`llama_model.rs`**: Llama.cpp execution configurations
  - `LlamaCppConfig` - Server configuration
  - `ServerMetrics` - CPU/GPU/RAM/VRAM metrics
  - `ModelState` - Runtime state
  
- **`manifest_model.rs`**: Model library and GGUF metadata
  - `ModelManifest` - GGUF metadata
  - `ModelInfo` - Model information
  - `ModelLibrary` - Collection of models
  
- **`mcp_model.rs`**: Model Context Protocol types
  - `McpConfig` - MCP server configuration
  - `McpServer` - Server definition
  - `McpTool` - Tool definition
  - `McpCapabilities` - Server capabilities

### Benefits

- **No Duplication**: Prevents redeclaring common structures like `ChatMessage` or `AppConfig` in multiple files
- **Type Safety**: Ensures that all layers (Commands → Services → Infrastructure) use the exact same types
- **Maintainability**: Changing a data field only requires updating one file in `models/`
- **Re-exports**: All models are re-exported at the module level for easy imports

## 2. Layered Architecture

The backend is divided into four distinct layers:

### 1. Commands (`/commands`)
Entry point for Tauri IPC. Bridge between Frontend and Service layers.

**Files:**
- `chat.rs` - Send chat messages
- `chat_actions.rs` - Copy, regenerate, edit messages
- `config.rs` - Load/save app configuration
- `general.rs` - General utility commands
- `llama_cpp.rs` - Start/stop llama server
- `mcp.rs` - MCP tool operations
- `mcp_config.rs` - MCP configuration
- `models.rs` - Model management

**Responsibilities:**
- Minimal logic, focusing on parameter translation
- Error conversion to frontend-friendly messages
- Tauri command attribute (`#[tauri::command]`)

### 2. Services (`/services`)
Business logic layer. Orchestrates operations, manages state, and interacts with external processes.

**Structure:**
```rust
pub mod services {
    pub mod llama {
        pub mod actor;    // Actor pattern for state management
        pub mod service;  // Public API
    }
    pub mod mcp {
        pub mod client;   // MCP client implementation
        pub mod protocol; // Protocol types
        pub mod service;  // MCP orchestration
    }
    pub mod capability_registry;  // Tauri capabilities
    pub mod orchestrator;         // Chat + tool calling
    pub mod subagent;            // Subagent spawning
    pub mod templates;           // Chat templates
    pub mod thinking_parser;     // Sequential thinking
}
```

**Responsibilities:**
- Implement business logic
- Manage application state (via Actor pattern)
- Coordinate between infrastructure and commands
- Handle async operations

### 3. Infrastructure (`/infrastructure`)
Low-level system interactions. Handles all "side effects" and OS interactions.

**Structure:**
```rust
pub mod infrastructure {
    pub mod llama {
        pub mod process;  // Process registry
        pub mod server;   // Server spawning, SSE streaming
    }
    pub mod metrics;      // CPU/RAM metrics
    pub mod nvidia_smi;   // GPU/VRAM metrics
}
```

**Responsibilities:**
- Spawn and manage external processes
- Handle SSE streaming from llama-server
- Collect system metrics
- File system operations
- Network communication

### 4. Models (`/models`)
Pure data structures. No logic besides initialization or basic formatting.

**Responsibilities:**
- Define data structures
- Implement `Serialize` and `Deserialize` for IPC
- Implement `Display`, `Debug` for debugging
- No business logic

## 3. State Management

### Global State (`state.rs`)

```rust
pub struct AppState {
    pub models_path: PathBuf,
    pub llama_service: Arc<LlamaCppService>,
    pub mcp_service: Arc<McpService>,
    pub orchestrator: Arc<Orchestrator>,
    pub mcp_config: Arc<RwLock<McpConfig>>,
}
```

Managed via Tauri's state management:
```rust
app.manage(AppState::new(models_path, mcp_config));
```

### Actor Pattern

Used in `services/llama/actor.rs` for thread-safe state management:
- Single source of truth for llama-server state
- Message-passing via `mpsc` channels
- Prevents race conditions
- Encapsulates mutable state

## 4. IPC Configuration

All Tauri commands are registered in `ipc_handlers.rs`:

```rust
pub fn configure_ipc(builder: tauri::Builder) -> tauri::Builder {
    builder.invoke_handler(tauri::generate_handler![
        // Chat commands
        commands::chat::send_message,
        commands::chat_actions::copy_message,
        // ... all other commands
    ])
}
```

## 5. Implementation Guidelines

### Imports
```rust
// ✅ Good - Import from models module
use crate::models::{ChatMessage, AppConfig};

// ❌ Bad - Don't import from specific model files
use crate::models::chat_model::ChatMessage;
```

### Serialization
- All models exposed to frontend MUST implement `Serialize` and `Deserialize`
- Non-serializable types (e.g., `tokio::process::Child`) belong in infrastructure or services
- Use `#[serde(skip)]` for non-serializable fields if needed

### Error Handling
```rust
// Commands should return Result<T, String>
#[tauri::command]
pub async fn my_command() -> Result<MyResponse, String> {
    service.do_something()
        .await
        .map_err(|e| e.to_string())
}
```

### Async Operations
- Use `tokio::spawn` for background tasks
- Use `Arc` for shared state across threads
- Use `RwLock` or `Mutex` for mutable shared state
- Prefer message-passing (Actor pattern) over shared mutable state

## 6. Module Organization

Each layer follows this pattern:

```
src-tauri/src/
├── lib.rs              # Module declarations, Tauri setup
├── main.rs             # Entry point
├── state.rs            # Global state
├── utils.rs            # Utility functions
├── ipc_handlers.rs     # Command registration
├── commands/           # IPC handlers (8 files)
├── services/           # Business logic (10 files)
├── infrastructure/     # System interactions (4 files)
└── models/             # Data structures (5 files)
```

## 7. Testing Strategy

- **Unit Tests**: Test individual functions in services and infrastructure
- **Integration Tests**: Test command → service → infrastructure flow
- **Mock Infrastructure**: Use traits to mock external dependencies
- **Test Files**: Located in `src-tauri/tests/`

---

*Follow these rules to ensure the codebase remains scalable, maintainable, and predictable.*

*Last updated: 2026-03-28*
