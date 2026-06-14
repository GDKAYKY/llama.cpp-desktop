# File Responsibilities & Module Ownership

This document provides a detailed breakdown of the responsibilities for each major file in the backend source tree.

## 1. Infrastructure Layer (`src-tauri/src/infrastructure`)
*The "Interface" to the outside world. No business state, only IO.*

- **`gpu/hardware_metrics.rs`**: 
    - **Owner**: Centralized Hardware Metrics Collection
    - **Logic**: Aggregates CPU/RAM/GPU metrics from multiple platform-specific sources.
    - **Components**:
        - `SystemMetricsProvider`: Provides CPU usage (%) and RAM usage (bytes) via `sysinfo` crate.
        - `NvmlMetricsProvider`: NVIDIA GPU metrics (VRAM, utilization, temperature) via NVIDIA NVML SDK (Windows only).
        - `PdhMetricsProvider`: Windows GPU metrics (VRAM, utilization) via Windows PDH (Performance Data Helper) counters.
        - `WgpuDetector`: GPU enumeration (name, vendor, index) via wgpu, includes integrated GPUs and non-NVIDIA cards.
        - `FallbackGpuMetricsProvider`: Tries NVML → PDH → wgpu in fallback order for complete coverage.
        - `MetricsProvider` trait: Single interface for all metrics snapshotting.
    - **Responsibilities**: 
        - Produce `ServerMetrics` snapshots (CPU, RAM, GPU, VRAM) for a given PID.
        - Automatically select and compose best available metrics provider for the platform.
        - Handle platform-specific APIs (Windows PDH, NVIDIA NVML) transparently.

- **`gpu/mod.rs`**:
    - **Owner**: Module API Surface
    - **Responsibilities**: Re-export public types and providers from `hardware_metrics.rs`.


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
- **`chat_actions.rs`**: Maps message action commands (like, dislike, share, regenerate) to backend logic.
- **`models.rs`**: Handles model library scanning and manifest parsing.
- **`config.rs`**: Manages application-wide settings (stored in JSON).
- **`mcp.rs`**: MCP server lifecycle commands (connect, list, tools/resources).
- **`mcp_config.rs`**: MCP config file load/save/reset commands.

## 4. Models Layer (`src-tauri/src/models`)
*The "Language". Data structures used across all layers.*

- **`llama.rs`**: Definitions for `LlamaCppConfig`, `ServerMetrics`, `ModelId`, and `ModelState`.
- **`chat.rs`**: Definitions for `ChatMessage` and `ChatRequest`.
- **`app_config.rs`**: Configuration schema for the entire app.

## 5. State Layer (`src-tauri/src/state`)
*The "Container". Dependency injection and global state.*

- **`state.rs`**: Initializes and holds instances of all services (`LlamaCppService`, `Orchestrator`, `McpService`).

## 6. Binding Layer (`src-tauri/src`)
*Connects the layers and handles the IPC registry.*

- **`ipc_handlers.rs`**:
    - **Owner**: IPC Management
    - **Logic**: Registry of all `#[tauri::command]` functions.
    - **Responsibilities**: Centralizing command registration to keep `lib.rs` and `main.rs` clean.

