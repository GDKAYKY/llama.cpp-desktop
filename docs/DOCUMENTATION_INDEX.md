# Documentation Index

> **Complete documentation for Llama Desktop - A Tauri + Rust + Svelte desktop app for running local LLMs**

## 📋 Quick Navigation

- [Project Structure](./architecture/PROJECT_STRUCTURE.md) - Complete file and directory reference
- [High-Level Architecture](./architecture/HIGH_LEVEL_ARCHITECTURE.md) - System overview
- [Getting Started](#-getting-started) - Setup and configuration guides
- [Backend Documentation](#-backend-rust--tauri) - Rust/Tauri implementation
- [Frontend Documentation](#-frontend-svelte) - Svelte/TypeScript implementation

---

## 🏗️ Architecture & Overview

### System Architecture
- **[High-Level Architecture](./architecture/HIGH_LEVEL_ARCHITECTURE.md)** - Birds-eye view of system components and interactions
- **[Architecture Diagrams](./architecture/ARCHITECTURE_DIAGRAMS.md)** - Visual diagrams (Mermaid) of system architecture
- **[Data Flows](./architecture/DATA_FLOWS.md)** - Complete data flow documentation
- **[Project Structure](./architecture/PROJECT_STRUCTURE.md)** - Complete directory and file organization (200+ files documented)
- **[File Responsibilities](./architecture/FILE_RESPONSIBILITIES.md)** - Breakdown of what each major file does

---

## 🚀 Getting Started

### Setup & Configuration
- **[Models Setup Guide](./guides/MODELS_SETUP_GUIDE.md)** - How to acquire and organize GGUF models
- **[Configuration Guide](./guides/CONFIGURATION_GUIDE.md)** - Application configuration options
- **[Model Path Configuration](./guides/MODEL_PATH.md)** - Details on how the app locates models
- **[Release Process](./guides/RELEASE_PROCESS.md)** - Steps to build and package the application

---

## 🦀 Backend (Rust + Tauri)

### Architecture & Patterns
- **[Backend Architecture](./backend/BACKEND_ARCHITECTURE.md)** - Standards and conventions for the Rust backend
  - Layered architecture (Commands → Services → Infrastructure)
  - Centralized data models
  - Actor pattern for state management
  - Separation of concerns

### Core Services

#### Llama.cpp Integration
- **[Llama.cpp Integration](./backend/LLAMA_CPP_INTEGRATION.md)** - How to configure and use the llama.cpp backend
  - Process management
  - Server lifecycle
  - SSE streaming
  - Metrics collection
- **[Llama.cpp Request Parameters](./backend/LLAMA_CPP_REQUEST_PARAMETERS.md)** - Complete reference for request parameters
  - Sampling parameters
  - Context settings
  - Performance tuning

#### Model Context Protocol (MCP)
- **[MCP Servers](./backend/MCP_SERVERS.md)** - Configuration and management of MCP servers
  - Server setup
  - Stdio communication
  - Tool discovery
- **[MCP Tool Calling](./backend/MCP_TOOL_CALLING.md)** - Implementation details for MCP tool calling
  - Protocol implementation
  - Tool execution flow
  - Error handling
  - Streaming with tool calls

#### Model Management
- **[Model Parsing](./backend/MODEL_PARSING_README.md)** - Technical details on how GGUF manifests are parsed
  - GGUF metadata extraction
  - Model library management
  - Quantization detection

### Security & Permissions
- **[Tauri Capabilities](./backend/TAURI_CAPABILITIES.md)** - Security and IPC permissions configuration
  - Capability system
  - IPC command permissions
  - File system access
  - Shell execution

### Backend File Structure
```
src-tauri/src/
├── commands/          # IPC command handlers (8 files)
├── models/            # Data structures (5 files)
├── services/          # Business logic (10 files)
│   ├── llama/        # Llama.cpp service
│   └── mcp/          # MCP service
├── infrastructure/    # System interactions (4 files)
│   └── llama/        # Process management
├── main.rs           # Entry point
├── lib.rs            # Module declarations
├── state.rs          # Global state
└── utils.rs          # Utilities
```

---

## 🎨 Frontend (Svelte)

### UI & Design
- **[UI & Design](./frontend/UI_AND_DESIGN.md)** - Design principles, aesthetics, and branding logic
  - Color scheme
  - Component patterns
  - Responsive design
- **[Chat Pill Headers](./frontend/CHAT_PILL_HEADERS.md)** - Chat message header implementation
  - Message formatting
  - Role indicators
  - Timestamp display

### Features & Services

#### Chat History
- **[Chat History & Context](./frontend/CHAT_HISTORY.md)** - Persistent storage and context retrieval
  - IndexedDB schema
  - Hybrid context retrieval
  - Keyword extraction
  - Scoring algorithm
  - Performance optimization

### Frontend File Structure
```
src/
├── components/        # UI components (22 files)
│   ├── app/          # Application-level
│   ├── chat/         # Chat interface
│   ├── layout/       # Layout components
│   └── ui/           # Reusable UI
├── lib/              # Core library (31 files)
│   ├── config/       # Configuration
│   ├── services/     # Business logic
│   ├── stores/       # State management (6 stores)
│   ├── types/        # TypeScript types
│   └── markdown/     # Markdown processing
├── pages/            # Full pages (5 files)
└── routes/           # SvelteKit routes (7 files)
```

---

## 📚 Reference

### Developer Resources
- **[Developer Examples](./DEVELOPER_EXAMPLES.md)** - Practical code examples for common tasks
  - Adding Tauri commands
  - Creating services
  - Frontend components
  - Testing examples

### Templates & Examples
- **[Llama Config Template](./LLAMA_CONFIG_TEMPLATE.json)** - Configuration template for llama.cpp

### Changelog
- **[Documentation Changelog](./CHANGELOG_DOCS.md)** - Documentation update history

---

## 📊 Documentation Statistics

- **Total Files Documented**: 200+
- **Documentation Pages**: 19
- **Backend Files**: 32 Rust files
- **Frontend Files**: 66 TypeScript/Svelte files
- **Test Files**: 50+ test files
- **Last Updated**: 2026-03-28

---

## 🔗 External Resources

- [Main README](../README.md) - Project overview and installation
- [Agent Configuration](../AGENTS.md) - AI agent setup
- [llama.cpp Repository](https://github.com/ggml-org/llama.cpp) - Upstream project

---

## 📖 Documentation Organization

### By Role

**For Developers**
1. Start with [High-Level Architecture](./architecture/HIGH_LEVEL_ARCHITECTURE.md)
2. Review [Project Structure](./architecture/PROJECT_STRUCTURE.md)
3. Read [Backend Architecture](./backend/BACKEND_ARCHITECTURE.md) or [UI & Design](./frontend/UI_AND_DESIGN.md)
4. Dive into specific feature docs

**For Users**
1. [Models Setup Guide](./guides/MODELS_SETUP_GUIDE.md)
2. [Configuration Guide](./guides/CONFIGURATION_GUIDE.md)
3. [MCP Servers](./backend/MCP_SERVERS.md) (if using tools)

**For Contributors**
1. [Backend Architecture](./backend/BACKEND_ARCHITECTURE.md) - Code standards
2. [File Responsibilities](./architecture/FILE_RESPONSIBILITIES.md) - Where to make changes
3. [Release Process](./guides/RELEASE_PROCESS.md) - How to release

### By Technology

**Rust/Tauri Backend**
- Backend Architecture
- Llama.cpp Integration
- MCP Tool Calling
- Tauri Capabilities
- Model Parsing

**Svelte Frontend**
- UI & Design
- Chat History
- Chat Pill Headers

**Full Stack**
- High-Level Architecture
- Project Structure
- Configuration Guide

---

*Documentation maintained by the Llama Desktop team*
