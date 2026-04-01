# Data Flows

Complete documentation of data flows through the Llama Desktop application.

## 1. Chat Message Flow

```
User Input → ChatForm.svelte → chat.svelte.ts
    ↓
IPC: send_message({ message, conversationId })
    ↓
commands/chat.rs → Orchestrator::process_chat()
    ↓
LlamaServer HTTP POST /completion
    ↓
SSE Stream → Parse chunks → Detect tool calls
    ↓
[If tool call] → McpService::tools_call() → MCP Server
    ↓
Tool result → Continue streaming
    ↓
Frontend receives chunks → Update UI
```

## 2. Model Loading Flow

```
User clicks "Load Model"
    ↓
IPC: start_llama_server(config)
    ↓
LlamaCppService::start() → Actor receives Start message
    ↓
Check state → Spawn llama-server.exe
    ↓
Wait for "HTTP server listening"
    ↓
Update state to Running → Return success
    ↓
UI shows "Running" badge
```

## 3. MCP Tool Calling Flow

```
LLM generates <tool_call>
    ↓
Orchestrator parses XML
    ↓
McpService::tools_call(server_id, name, args)
    ↓
Check allowlist → Send to MCP server
    ↓
MCP executes tool → Return result
    ↓
Format as <tool_result>
    ↓
Append to context → Continue generation
```

## 4. Chat History Flow

```
Message received
    ↓
history.ts::saveMessage()
    ↓
Extract keywords → Estimate tokens
    ↓
IndexedDB: messages.add()
    ↓
Update conversation.updatedAt
```

### Context Retrieval

```
New message
    ↓
findRelevantContext(query)
    ↓
Extract keywords → Query IndexedDB
    ↓
Score candidates → Select top 5
    ↓
Format as context → Prepend to prompt
```

## 5. Configuration Flow

```
User changes setting
    ↓
IPC: save_config(config)
    ↓
Serialize to JSON
    ↓
Write to ~/.llama-desktop/config.json
    ↓
Return success → Show toast
```

---

*Last updated: 2026-03-28*
