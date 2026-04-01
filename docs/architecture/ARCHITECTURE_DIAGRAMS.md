# Architecture Diagrams

Visual representations of the Llama Desktop architecture.

## System Overview

```mermaid
graph TB
    subgraph "Frontend (Svelte)"
        UI[UI Components]
        Stores[Svelte Stores]
        Services[Frontend Services]
    end
    
    subgraph "Tauri Bridge"
        IPC[IPC Layer]
    end
    
    subgraph "Backend (Rust)"
        Commands[Commands Layer]
        BServices[Services Layer]
        Infrastructure[Infrastructure Layer]
        Models[Data Models]
    end
    
    subgraph "External"
        LlamaServer[llama-server.exe]
        MCPServers[MCP Servers]
        IndexedDB[(IndexedDB)]
    end
    
    UI --> Stores
    Stores --> Services
    Services --> IPC
    IPC --> Commands
    Commands --> BServices
    BServices --> Infrastructure
    BServices --> Models
    Infrastructure --> LlamaServer
    Infrastructure --> MCPServers
    Services --> IndexedDB
```

## Chat Flow with MCP Tools

```mermaid
sequenceDiagram
    participant User
    participant UI
    participant Backend
    participant Orchestrator
    participant LlamaServer
    participant MCPServer
    
    User->>UI: Send message
    UI->>Backend: send_message()
    Backend->>Orchestrator: process_chat()
    Orchestrator->>LlamaServer: POST /completion
    
    loop Streaming
        LlamaServer-->>Orchestrator: SSE chunk
        Orchestrator-->>Backend: Stream response
        Backend-->>UI: Update message
    end
    
    alt Tool Call Detected
        Orchestrator->>MCPServer: tools/call
        MCPServer-->>Orchestrator: Tool result
        Orchestrator->>LlamaServer: Continue with result
    end
```

## Model Loading Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Backend
    participant LlamaActor
    participant Process
    
    User->>Frontend: Click "Load Model"
    Frontend->>Backend: start_llama_server(config)
    Backend->>LlamaActor: Send Start message
    
    alt Already Running
        LlamaActor-->>Backend: Error
    else Not Running
        LlamaActor->>Process: Spawn llama-server.exe
        LlamaActor->>LlamaActor: Update state
        LlamaActor-->>Backend: Success
    end
    
    Backend-->>Frontend: Update UI
```

## State Management (Actor Pattern)

```mermaid
graph TB
    subgraph "LlamaService"
        API[Public Methods]
        Sender[Message Sender]
    end
    
    subgraph "Actor Thread"
        Receiver[Message Receiver]
        State[Mutable State]
        Handler[Message Handler]
    end
    
    API --> Sender
    Sender -->|mpsc channel| Receiver
    Receiver --> Handler
    Handler --> State
```

---

*Last updated: 2026-03-28*
