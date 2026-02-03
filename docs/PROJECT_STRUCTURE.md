# Llama.cpp Desktop Project Structure

## Overview

The Llama Desktop project is organized into three independent layers:

1. **Frontend Layer** (Svelte) - User interface running in Tauri webview
2. **Backend Layer** (Rust) - Core logic and model management
3. **Desktop Framework** (Tauri) - Desktop application lifecycle and IPC management

This architecture enables clear separation of concerns, performance, and cross-platform compatibility.

## Directory Structure

```txt
llama-desktop/
├─ src/                  # Svelte (frontend)
│  ├─ lib/               # Shared components and logic
│  │  ├─ services/       # Frontend-only logic (Dexie, history)
│  │  ├─ stores/         # Svelte stores (Chat, Server, Models)
│  │  ├─ infrastructure/ # IPC wrapper
│  │  └─ ...
│  ├─ components/        # UI components (ModelCard, Message, etc.)
│  ├─ pages/             # App pages (Home, Settings, Models)
│  └─ App.svelte         # Main entry component
│
├─ src-tauri/            # Rust (backend Tauri)
│  ├─ src/
│  │  ├─ main.rs         # Setup entry point
│  │  ├─ lib.rs          # Core Tauri application logic
│  │  ├─ commands/       # Tauri command handlers (per-domain files)
│  │  ├─ commands.rs     # Registry of command modules
│  │  ├─ models/         # Shared data structures
│  │  ├─ models.rs       # Registry of model modules
│  │  ├─ services/       # Core business logic (Actor services)
│  │  ├─ services.rs     # Registry of service modules
│  │  ├─ state/          # Global AppState (single state.rs)
│  │  ├─ ipc_handlers.rs  # Centralized IPC command registration
│  │  └─ infrastructure/ # IO and hardware management
│
├─ docs/                  # Project documentation (indexed by README.md)
├─ package.json           # Node dependencies & scripts
├─ tauri.conf.json        # Tauri configuration
└─ README.md

```

## Layer Descriptions

### Frontend Layer (src/)

The frontend is a Svelte-based user interface running in a Tauri webview.

**Key Files:**
- `src/main.js` - Entry point that initializes the Svelte app
- `src/App.svelte` - Root component
- `src/components/` - Reusable UI components
- `src/lib/ipc.js` - IPC communication wrapper for backend calls
- `src/lib/errorHandler.js` - Error handling utilities

**Key Technologies:**
- Svelte 5 - UI framework
- Vite - Build tool and dev server
- @tauri-apps/api - Tauri API access

**Development:**
```bash
npm run dev:frontend  # Start frontend dev server on port 5173
```

### Backend Layer (src-tauri/)

The backend is written in Rust and integrated directly into the Tauri application. It handles model execution, file system access, and system-level operations.

**Key Files:**
- **`src-tauri/src/services/`**: Modular logic separated into Actors and high-level services like Orchestrator.
- **`src-tauri/src/infrastructure/`**: Direct interactions with external binaries (`llama-server`) and hardware (`nvidia-smi`).
- **`src-tauri/src/commands/`**: Rust functions exposed as Tauri commands.
- **`src-tauri/src/state/`**: Global application state management.

**Key Technologies:**
- Rust - High-performance backend language
- Tauri - Desktop application framework
- Serde - Serialization and deserialization
- llama-cpp-rs - (Optional) Rust bindings for llama.cpp

#### Backend Models Standard
All shared data structures (Structs and Enums) must be placed in `src-tauri/src/models/`. 
- **Centralization**: Avoid defining structs inside `commands/` or `services/` if they are used by more than one module or if they represent core data entities.
- **Organization**: Models are grouped by domain (e.g., `chat.rs`, `llama.rs`, `manifest.rs`).
- **Cleanliness**: Services and Commands should only contain logic, importing their data structures from the `models` module.
- **Portability**: Data structures that represent API contracts or state should be easily serializable (using `serde`).

**Development:**
```bash
npm run dev  # Starts both Svelte frontend and Rust backend in dev mode
```

### Tauri Layer (src-tauri/)

Tauri bridges the frontend and backend, managing window lifecycles and secure IPC.

**Key Responsibilities:**
- Exposing Rust commands to the Svelte frontend
- Managing native window properties and menus
- Handling secure file system and system-level permissions
- Bundling the application for distribution

## IPC Communication

The frontend communicates with the Rust backend through Tauri's `invoke` system.

**Frontend to Backend (Svelte):**
```javascript
import { invoke } from '@tauri-apps/api/core';

async function sendMessage(message) {
  try {
    const response = await invoke('send_chat_message', { message });
    console.log('Backend response:', response);
  } catch (error) {
    console.error('IPC Error:', error);
  }
}
```

**Backend Handler (Rust):**
```rust
#[tauri::command]
fn send_chat_message(message: String) -> String {
    format!("Processed: {}", message)
}
```

## Development Workflow

### Setup

1. **Install Prerequisites**:
   - [Rust](https://rustup.rs/)
   - [Node.js](https://nodejs.org/)

2. **Install Dependencies**:
```bash
npm install
```

3. **Start Development Mode**:
```bash
npm run dev
```

This command starts:
- Frontend dev server (Vite)
- Tauri development environment with Rust auto-recompile

### Building

```bash
npm run build
```

This command:
1. Builds the frontend (Vite)
2. Installs backend dependencies
3. Builds the Tauri application

### Testing

```bash
npm test
```

This runs:
- Frontend tests (Jest)
- Backend tests (Jest)

### Linting

```bash
npm run lint          # Check for linting errors
npm run lint:fix      # Fix linting errors
```

## Adding New Features

### Adding a Chat Feature

1. **Backend (src-tauri/)**
   - Create a command in `src-tauri/src/commands/chat.rs`
   - Register the command in `main.rs`
   - Implement logic using Rust crates (e.g., `llama-cpp-rs`)

2. **Frontend (src/)**
   - Create a Svelte component in `src/components/`
   - Use `invoke('command_name')` from `@tauri-apps/api/core` to call the Rust backend

### Example: Adding a New Chat Command

**Backend (Rust):**
```rust
// src-tauri/src/commands/chat.rs
#[tauri::command]
pub async fn get_message_count() -> Result<u32, String> {
    // Logic to get count
    Ok(42)
}
```

**Frontend (Svelte):**
```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  let count = 0;
  
  async function loadCount() {
    count = await invoke('get_message_count');
  }
</script>
```

## Error Handling

### Frontend Error Handling

- Use `try/catch` around `invoke` calls.
- Display errors to users via Svelte state or toast notifications.

### Backend Error Handling

- Return `Result<T, E>` from Tauri commands.
- Tauri automatically converts `Result` to IPC-compatible error responses.
- Errors must implement `serde::Serialize`.

## Performance Considerations

1. **IPC Communication**: Keep messages small and avoid frequent calls
2. **Backend Processing**: Implement caching for frequently accessed data
3. **Frontend Rendering**: Use Svelte stores for state management
4. **Build Optimization**: Vite automatically optimizes frontend bundle

## Troubleshooting

## Troubleshooting

### Rust backend not compiling
- Ensure `rustc --version` shows a recent stable version.
- Check `src-tauri/Cargo.toml` for missing dependencies.
- Use `cargo check` in `src-tauri` for faster feedback.

### IPC communication failing
- Verify the command name in Rust (using `#[tauri::command]`) matches the string in `invoke()`.
- Ensure the command is registered in `generate_context!` within `main.rs`.
- Check the webview console (F12) for detailed error messages.

### Frontend not loading
- Check if Vite dev server is running.
- Ensure `tauri.conf.json` points to the correct `devPath`.

## Resources

- [Tauri Documentation (v2)](https://v2.tauri.app/)
- [Svelte Documentation](https://svelte.dev/)
- [Rust Programming Language](https://www.rust-lang.org/)
- [Vite Documentation](https://vitejs.dev/)
