# Backend Architecture Standards

This document defines the mandatory code organization standards for the Rust backend in the Llama.cpp Desktop project.

## 1. Centralized Data Models

**Rule:** All shared data structures, including Structs, Enums, and Constants that represent data entities, must reside in `src-tauri/src/models/`.

### Directory Layout
- `src-tauri/src/models/mod.rs`: Exports all sub-modules.
- `src-tauri/src/models/app_config.rs`: Global application settings.
- `src-tauri/src/models/chat.rs`: API contracts and state for chat sessions.
- `src-tauri/src/models/llama.rs`: Llama-specific execution configurations and runtime state types.
- `src-tauri/src/models/manifest.rs`: Model library and OCI manifest structures.

### Benefits
- **No Duplication**: Prevents redeclaring common structures like `ChatMessage` or `AppConfig` in multiple files.
- **Type Safety**: Ensures that all layers (Commands -> Services -> Actors) use the exact same types.
- **Maintainability**: Changing a data field only requires updating one file in `models/`.

## 2. Separation of Responsibilities

The backend is divided into three distinct layers:

1.  **Commands (`/commands`)**: Entry point for Tauri IPC. Bridge between Frontend and Service layers. Should contain minimal logic, focusing on parameter translation and error conversion.
2.  **Services (`/services`)**: Business logic. Orchestrates operations, manages state, and interacts with external processes/APIs.
3.  **Models (`/models`)**: Pure data structures. No logic besides initialization or basic formatting (e.g., `Display` implementation).

## 3. Implementation Guidelines

- **Clean Imports**: Use `use crate::models::{...};` to import types.
- **Internal States**: Even internal runtime states (like `ModelState`) should be defined in the appropriate domain file in `models/`.
- **Serialization**: Most models should implement `Serialize` and `Deserialize` to be compatible with Tauri's IPC. (Exception: Models containing non-serializable types like `tokio::process::Child`).
    - Runtime state must remain serializable. If a type requires non-serializable handles, store those in infrastructure or services, not in `models/`.

---
*Follow these rules to ensure the codebase remains scalable and predictable.*
