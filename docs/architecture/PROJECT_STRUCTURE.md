> **Note:** This document provides a comprehensive overview of the project's directory structure and key files. For detailed architecture information, see [BACKEND_ARCHITECTURE](../backend/BACKEND_ARCHITECTURE.md) and [HIGH_LEVEL_ARCHITECTURE](./HIGH_LEVEL_ARCHITECTURE.md).

## Root Directory

```
llama.cpp-desktop/
├── src/                    # Frontend (Svelte)
├── src-tauri/              # Backend (Rust + Tauri)
├── docs/                   # Documentation
├── tests/                  # Frontend tests
├── static/                 # Static assets
├── .github/                # CI/CD workflows
├── .agent/                 # AI agent rules
├── subagent/               # Subagent implementation examples
├── .svelte-kit/            # SvelteKit build output (generated)
├── node_modules/           # Node.js dependencies (generated)
├── README.md               # Project overview, installation, features
├── AGENTS.md               # AI agent configuration
├── LICENSE                 # Project license (MIT, Apache, etc.)
├── package.json            # Node.js dependencies and scripts
├── package-lock.json       # Locked dependency versions
├── vite.config.js          # Vite bundler configuration
├── svelte.config.js        # Svelte compiler configuration
├── jsconfig.json           # JavaScript/TypeScript configuration
├── tailwind.config.js      # Tailwind CSS configuration
├── postcss.config.js       # PostCSS configuration
├── .gitignore              # Git ignored files
├── sonar-project.properties # SonarCloud configuration
└── update_icons.py         # Icon generation script
```

## Frontend Structure (`src/`)

### Components (`src/components/`)
- **`app/`** - Application-level components
  - `ServerControl.svelte` - Start/stop llama server, display status, resource usage
  - `index.js` - Component exports
  
- **`chat/`** - Chat interface components
  - `ChatForm.svelte` - Message input with textarea, send button, file attachments
  - `ChatMessage.svelte` - Single message display (user/assistant), markdown rendering, code highlighting
  - `ChatMessages.svelte` - Scrollable message list, auto-scroll, loading states
  - `ChatMessageWindow.svelte` - Message container with header, manages scroll behavior
  - `ChatOrchestrator.svelte` - Orchestrates chat flow, handles MCP tool calls, streaming
  - `ModelUsageGraph.svelte` - Real-time CPU/GPU/RAM/VRAM usage charts
  
- **`layout/`** - Layout and navigation components
  - `ChatHeader.svelte` - Page title, model selector, settings button
  - `ChatSidebar.svelte` - Chat history list, new chat button, search
  - `ModelCard.svelte` - Model info card (name, size, parameters, quantization)
  - `ModelLogo.svelte` - Model logo/icon display with fallback
  - `ModelSelector.svelte` - Dropdown to select active model
  
- **`ui/`** - Reusable UI components
  - `Checkbox.svelte` - Custom styled checkbox
  - `Dropdown.svelte` - Dropdown menu with keyboard navigation
  - `MarkdownContent.svelte` - Markdown renderer with syntax highlighting, LaTeX support
  - `MessageAvatar.svelte` - User/AI avatar with initials or icon
  - `Modal.svelte` - Modal dialog with backdrop, close button
  - `TextShimmer.svelte` - Skeleton loading shimmer effect
  - `TypingIndicator.svelte` - Animated typing dots

### Library (`src/lib/`)
- **`config/`** - Application configuration
  - `AppConfig.ts` - Configuration type definitions
  - `defaultConfig.ts` - Default configuration values
  - `index.ts` - Config module exports
  
- **`constants/`** - Shared constants
  - `latex-protection.js` - Protect LaTeX syntax during markdown processing
  - `literal-html.js` - Handle literal HTML in markdown
  - `table-html-restorer.js` - Restore HTML tables after sanitization
  
- **`infrastructure/`** - Low-level utilities
  - `ipc.ts` - Tauri IPC wrapper (invoke commands, listen to events)
  
- **`markdown/`** - Markdown processing pipeline
  - `enhance-code-blocks.js` - Add copy button, language labels to code blocks
  - `enhance-links.js` - Process and enhance markdown links
  - `literal-html.js` - Preserve literal HTML during processing
  - `sanitize-schema.js` - DOMPurify sanitization rules
  - `table-html-restorer.js` - Restore table HTML after sanitization
  - `__tests__/sanitize.test.js` - Sanitization tests
  
- **`services/`** - Business logic services
  - `chat_templates.ts` - Load/apply chat templates (ChatML, Llama3, Mistral, etc.)
  - `history.ts` - Chat history persistence (IndexedDB via Dexie)
  - `models.ts` - Model list, metadata, selection
  - `models_delete.ts` - Model deletion with confirmation
  - `model_downloads.ts` - Download models from HuggingFace, track progress
  - `orchestrator.ts` - Frontend chat orchestration, MCP tool handling
  
- **`shared/`** - Shared utilities
  - `clipboard.js` - Copy to clipboard with fallback
  - `cn.js` - Tailwind class name merger (clsx + tailwind-merge)
  - `latex-protection.js` - LaTeX protection utilities
  
- **`stores/`** - Svelte stores (reactive state management)
  - `chat.svelte.ts` - Chat messages, active chat, streaming state
  - `mcp.svelte.ts` - MCP servers list, tools, connection status
  - `models.svelte.ts` - Available models, selected model, download progress
  - `server.svelte.ts` - Llama server status, metrics, errors
  - `settings.svelte.ts` - App settings (theme, models path, API keys)
  - `ui.svelte.ts` - UI state (sidebar open, modal state, notifications)
  
- **`types/`** - TypeScript type definitions
  - `backend.d.ts` - Backend API types (matches Rust types)
  - `models.ts` - Frontend model types

### Pages (`src/pages/`)
Full-page Svelte components:
- `Home.svelte` - Main chat interface (messages, input, sidebar)
- `Models.svelte` - Model management (list, download, delete, metadata)
- `Settings.svelte` - App settings (models path, server config, UI preferences)
- `Mcps.svelte` - MCP server configuration (add, edit, remove servers)
- `Customization.svelte` - UI customization (theme, colors, fonts)

### Routes (`src/routes/`)
SvelteKit routing structure:
- `+layout.svelte` - Root layout (navigation, global styles)
- `+layout.js` - Layout load function (preload data)
- `+page.svelte` - Home route (redirects to Home.svelte)
- `customization/+page.svelte` - Customization route
- `mcps/+page.svelte` - MCPs route
- `models/+page.svelte` - Models route
- `settings/+page.svelte` - Settings route

### Root Files
- `app.html` - HTML template (head, body, %sveltekit.head%, %sveltekit.body%)
- `app.css` - Global styles (Tailwind directives, custom CSS)
- `errorHandler.ts` - Global error handler (catch unhandled errors)
- `llama.ts` - Llama.cpp integration utilities (format prompts, parse responses)

## Backend Structure (`src-tauri/src/`)

### Commands (`src-tauri/src/commands/`)
Tauri IPC command handlers (Frontend → Backend bridge):
- `chat.rs` - Send chat messages, handle streaming responses
- `chat_actions.rs` - Copy message, regenerate response, edit message
- `config.rs` - Load/save app configuration (models directory, settings)
- `general.rs` - General utility commands (ping, version info)
- `llama_cpp.rs` - Start/stop llama server, get metrics, check status
- `mcp.rs` - List/call MCP tools, get server status
- `mcp_config.rs` - Load/save MCP server configuration
- `models.rs` - List/download/delete models, parse GGUF metadata

### Models (`src-tauri/src/models/`)
Data structures and types (all serializable for IPC):
- `app_settings_model.rs` - `AppConfig`, `AppSettings` (models directory, UI preferences)
- `chat_model.rs` - `ChatMessage`, `ChatRequest`, `ChatResponse`, `ToolCall`, `ToolResult`
- `llama_model.rs` - `LlamaCppConfig`, `ServerMetrics`, `ModelState`
- `manifest_model.rs` - `ModelManifest`, `ModelInfo`, `ModelLibrary` (GGUF metadata)
- `mcp_model.rs` - `McpConfig`, `McpServer`, `McpTool`, `McpCapabilities`

### Services (`src-tauri/src/services/`)
Business logic layer:
- **`llama/`** - Llama.cpp service (Actor pattern)
  - `actor.rs` - `LlamaActor` - Thread-safe state management, process lifecycle
  - `service.rs` - `LlamaCppService` - Public API (start, stop, send_message, get_metrics)
  
- **`mcp/`** - Model Context Protocol
  - `client.rs` - `McpClient` - Stdio communication with MCP servers
  - `protocol.rs` - MCP protocol types (JSON-RPC 2.0)
  - `service.rs` - `McpService` - Server lifecycle, tool discovery, tool execution
  
- `capability_registry.rs` - `CapabilityRegistry` - Dynamic Tauri capability management
- `orchestrator.rs` - `Orchestrator` - Chat flow with MCP tool calling, streaming
- `subagent.rs` - `SubagentService` - Subagent spawning and management
- `templates.rs` - `TemplateService` - Chat template formatting (ChatML, Llama3, etc.)
- `thinking_parser.rs` - `ThinkingParser` - Parse sequential-thinking MCP output

### Infrastructure (`src-tauri/src/infrastructure/`)
Low-level system interactions:
- **`llama/`** - Llama.cpp process management
  - `process.rs` - `ProcessRegistry` - Track running llama-server processes
  - `server.rs` - `LlamaServer` - Spawn llama-server.exe, handle SSE streaming
  
- `metrics.rs` - `get_system_metrics()` - CPU/RAM usage via sysinfo
- `nvidia_smi.rs` - `get_gpu_metrics()` - GPU/VRAM usage via nvidia-smi

### Root Files
- `main.rs` - Application entry point (calls `lib::run()`)
- `lib.rs` - Module declarations, Tauri setup, plugin initialization, state management
- `state.rs` - `AppState` - Global shared state (models path, services, MCP config)
- `utils.rs` - Utility functions (path resolution, string formatting)
- `ipc_handlers.rs` - `configure_ipc()` - Register all Tauri commands

## Tests

### Frontend Tests (`tests/`)
- **`config/`**
  - `index.test.ts` - Config loading, validation, defaults
  
- **`services/`**
  - `models.test.ts` - Model list, selection, metadata parsing
  - `orchestrator.test.ts` - Chat orchestration, tool calling flow
  
- **`stores/`**
  - `chat.test.ts` - Chat store (messages, streaming, state)
  - `mcp.test.ts` - MCP store (servers, tools, status)
  - `models.test.ts` - Models store (list, selection, downloads)
  - `server.test.ts` - Server store (status, metrics, errors)
  - `settings.test.ts` - Settings store (load, save, validation)
  - `ui.test.ts` - UI store (sidebar, modals, notifications)
  
- `history.test.ts` - Chat history (IndexedDB operations, search)
- `setup.ts` - Test setup (mocks, global config)

### Backend Tests (`src-tauri/tests/`)
- **Config & State**
  - `app_config.rs` / `app_config_test.rs` - Config loading, defaults, validation
  - `state_test.rs` - AppState initialization, shared state access
  
- **Chat**
  - `chat_actions.rs` / `chat_actions_test.rs` / `chat_actions_commands_test.rs` - Copy, regenerate, edit
  - `chat_commands_test.rs` - Chat IPC commands
  - `chat_models.rs` - ChatMessage, ChatRequest serialization
  - `chat_parsing.rs` - Parse chat responses, extract tool calls
  
- **Llama.cpp**
  - `llama_actor_test.rs` - Actor state management, lifecycle
  - `llama_cpp_commands_test.rs` - Start/stop server commands
  - `llama_model_test.rs` - LlamaCppConfig, ServerMetrics types
  - `llama_server_test.rs` - Server spawning, SSE streaming
  - `llama_service_test.rs` - Service API (start, stop, send_message)
  
- **MCP**
  - `mcp_client_test.rs` - MCP client (stdio communication)
  - `mcp_commands_test.rs` - MCP IPC commands
  - `mcp_config.rs` / `mcp_config_commands_test.rs` - MCP config load/save
  - `mcp_integration.rs` - End-to-end MCP integration
  - `mcp_service_test.rs` / `mcp_service_internal_test.rs` - MCP service logic
  - `mcp_stdio.rs` - Stdio protocol implementation
  
- **Models**
  - `model_integration.rs` - Model download, parsing, loading
  - `model_library.rs` - Model library management
  - `models_commands.rs` / `models_commands_test.rs` - Model IPC commands
  - `models_internal_test.rs` - Internal model logic
  
- **Infrastructure**
  - `metrics_test.rs` - CPU/RAM metrics collection
  - `nvidia_smi_test.rs` - GPU metrics parsing
  - `process_manager_test.rs` - Process lifecycle management
  
- **Utilities**
  - `utils.rs` / `utils_test.rs` - Utility functions, path resolution
  
- **IPC**
  - `ipc/` - IPC handler tests (command registration, error handling)
  
- `config_commands_test.rs` - Config IPC commands

## Documentation (`docs/`)

### Architecture
- `BACKEND_ARCHITECTURE.md` - Backend architecture standards
- `high-level-architecture.md` - System overview

### Features
- `CHAT_HISTORY.md` - Chat history implementation
- `LLAMA_CPP_INTEGRATION.md` - Llama.cpp integration guide
- `LLAMA_CPP_REQUEST_PARAMETERS.md` - Request parameters reference
- `MCP_SERVERS.md` - MCP server configuration
- `MCP_TOOL_CALLING.md` - MCP tool calling implementation
- `TAURI_CAPABILITIES.md` - Tauri capabilities configuration

### Guides
- `CONFIGURATION_GUIDE.md` - Configuration options
- `MODELS_SETUP_GUIDE.md` - Model setup instructions
- `MODEL_PATH.md` - Model path configuration
- `MODEL_PARSING_README.md` - Model parsing logic
- `RELEASE_PROCESS.md` - Release workflow
- `UI_AND_DESIGN.md` - UI design guidelines
- `chat-pill-headers.md` - Chat message headers

### Templates
- `llama_config_template.json` - Llama.cpp config template

### Meta
- `README.md` - Documentation index
- `file-responsibilities.md` - File responsibility mapping
- `PROJECT_STRUCTURE.md` - This file

## Configuration Files

### Build & Development
- **Node.js / Frontend**
  - `package.json` - Dependencies (Svelte, Vite, Tailwind, Dexie, etc.), scripts
  - `package-lock.json` - Locked dependency versions
  - `vite.config.js` - Vite bundler config (plugins, build options)
  - `svelte.config.js` - Svelte compiler config (preprocessors, adapters)
  - `jsconfig.json` - JavaScript/TypeScript config (paths, module resolution)
  - `tailwind.config.js` - Tailwind CSS config (theme, plugins, content paths)
  - `postcss.config.js` - PostCSS config (Tailwind, autoprefixer)

- **Rust / Backend**
  - `src-tauri/Cargo.toml` - Rust dependencies, package metadata, features
  - `src-tauri/Cargo.lock` - Locked Rust dependency versions
  - `src-tauri/build.rs` - Build script (compile-time code generation)
  - `src-tauri/.env` - Environment variables (API keys, paths)
  - `src-tauri/.gitignore` - Ignored files (target/, .env)

- **Tauri**
  - `src-tauri/tauri.conf.json` - Tauri config (app metadata, window settings, permissions)
  - `src-tauri/capabilities/default.json` - Default capability set (IPC permissions)
  - `src-tauri/gen/schemas/` - Generated JSON schemas for capabilities

### CI/CD
- `.github/workflows/rust.yml` - Rust CI (build, test, clippy, fmt)
- `.github/workflows/sonarcloud.yml` - SonarCloud code quality analysis
- `sonar-project.properties` - SonarCloud project configuration

### IDE
- `.vscode/settings.json` - VS Code workspace settings (formatters, linters)
- `.vscode/extensions.json` - Recommended VS Code extensions
- `.vscode/launch.json` - Debug configurations (Rust, Tauri, Node)

### AI Agents
- `AGENTS.md` - Agent configuration (MCP servers, capabilities)
- `.agent/rules/document.md` - Documentation generation rules

### Git
- `.gitignore` - Ignored files (node_modules/, target/, .env, build artifacts)

## Static Assets (`static/`)
- `favicon.ico` - Browser favicon (ICO format)
- `favicon.png` - Browser favicon (PNG format, 512x512)
- `favicon.svg` - Browser favicon (SVG, scalable)
- `logo.png` - Application logo (used in UI)
- `svelte.svg` - Svelte framework logo
- `tauri.svg` - Tauri framework logo
- `vite.svg` - Vite bundler logo

## Icons (`src-tauri/icons/`)
Platform-specific application icons (generated from source):
- **Cross-platform**
  - `icon.png` - Base icon (1024x1024)
  - `icon.svg` - Vector source icon
  
- **macOS**
  - `icon.icns` - macOS app icon bundle
  
- **Windows**
  - `icon.ico` - Windows app icon
  - `setup.ico` - Installer icon
  - `Square*.png` - Windows Store tiles (30x30 to 310x310)
  - `StoreLogo.png` - Windows Store logo
  
- **Linux**
  - `32x32.png`, `128x128.png`, `128x128@2x.png` - Various sizes

## Subagent (`subagent/`)
Example implementations and documentation for subagent pattern:
- `subagent.md` - Subagent pattern documentation
- `subagent.rs` - Subagent implementation example
- `subagent_flow.md` - Subagent execution flow diagram
- `integration_guide.rs` - Integration guide with examples
- `example_trace.rs` - Example execution trace

## Build Artifacts (Generated, not in Git)
- `.svelte-kit/` - SvelteKit build output
  - `generated/` - Generated client/server code
  - `output/` - Production build
  - `types/` - Generated TypeScript types
  
- `src-tauri/target/` - Rust build output
  - `debug/` - Debug builds
  - `release/` - Release builds
  
- `src-tauri/gen/` - Generated Tauri files
  - `schemas/` - JSON schemas for capabilities
  
- `node_modules/` - Node.js dependencies
- `dist/` - Final distribution build
