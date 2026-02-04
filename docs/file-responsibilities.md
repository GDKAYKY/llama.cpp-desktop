# File Responsibilities & Module Ownership

This document provides a detailed breakdown of the responsibilities for each major file in the backend source tree.

## 1. Infrastructure Layer (`src-tauri/src/infrastructure`)
*The "Interface" to the outside world. No business state, only IO.*

- **`nvidia_smi.rs`**: 
    - **Owner**: Hardware Telemetry
    - **Logic**: Executes `nvidia-smi`, parses CSV output.
    - **Responsibilities**: Converting external command results into usable Rust types (`f32` percentages).

- **`metrics.rs`**:
    - **Owner**: Metrics Snapshotting
    - **Logic**: Uses `sysinfo` for CPU/RAM and `nvidia-smi` for GPU/VRAM.
    - **Responsibilities**: Produces `ServerMetrics` snapshots for a given PID.

- **`llama/process.rs`**:
    - **Owner**: Process Registry
    - **Logic**: Tracks running `Child` handles by `ModelId`.
    - **Responsibilities**: Register/remove process handles and expose PID lookups.

- **`llama/server.rs`**:
    - **Owner**: Process Lifecycle
    - **Logic**: Process spawning, log piping, HTTP streaming (reqwest).
    - **Responsibilities**: Validating binary paths, ensuring `llama-server.exe` starts correctly, and handling the low-level HTTP stream parsing.

## 2. Service Layer (`src-tauri/src/services`)
*The "Brain" of the application. Manages state and complex flows.*

- **`llama/actor.rs`**:
    - **Owner**: Llama Service State
    - **Logic**: Actor Message Loop (Enum-based dispatch).
    - **Responsibilities**: Maintaining `ModelState`, coordinating lifecycle via `ProcessManager`, and querying `MetricsProvider`.

- **`llama/service.rs`**:
    - **Owner**: Public API
    - **Logic**: Channel sender wrapper.
    - **Responsibilities**: Exposing a clean, async API (`start()`, `stop()`, `send_chat()`) to Tauri commands without exposing actor internals.

- **`orchestrator.rs`**:
    - **Owner**: Chat Session Management
    - **Logic**: History persistence (in-memory) and flow coordination.
    - **Responsibilities**: Combining user input with history, calling the `LlamaCppService`, and piping events back to the UI.

## 3. Command Layer (`src-tauri/src/commands`)
*The "Gateway". Logic-thin handlers for Tauri IPC.*

- **`llama_cpp.rs`**: Maps model-related Tauri commands to `LlamaCppService`.
- **`chat.rs`**: Maps chat-related Tauri commands to the `Orchestrator`.
- **`models.rs`**: Handles model library scanning and manifest parsing.
- **`config.rs`**: Manages application-wide settings (stored in JSON).

## 4. Models Layer (`src-tauri/src/models`)
*The "Language". Data structures used across all layers.*

- **`llama.rs`**: Definitions for `LlamaCppConfig`, `ServerMetrics`, `ModelId`, and `ModelState`.
- **`chat.rs`**: Definitions for `ChatMessage` and `ChatRequest`.
- **`app_config.rs`**: Configuration schema for the entire app.

## 5. State Layer (`src-tauri/src/state`)
*The "Container". Dependency injection and global state.*

- **`mod.rs`**: Initializes and holds instances of all services (`LlamaCppService`, `Orchestrator`).

## 6. Binding Layer (`src-tauri/src`)
*Connects the layers and handles the IPC registry.*

- **`ipc_handlers.rs`**:
    - **Owner**: IPC Management
    - **Logic**: Registry of all `#[tauri::command]` functions.
    - **Responsibilities**: Centralizing command registration to keep `lib.rs` and `main.rs` clean.

