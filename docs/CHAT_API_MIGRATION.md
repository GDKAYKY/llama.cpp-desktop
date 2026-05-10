# Chat API Migration Summary

## Overview

The TypeScript chat API interfaces have been updated to match the Rust backend implementation and llama.cpp's OpenAI-compatible `/v1/chat/completions` endpoint.

## What Changed

### 1. ChatRequest Interface (Breaking Change)

**Before:**
```typescript
interface ChatRequest {
    messages: Message[];
    tools?: ToolSpec[];
    params: GenerationParams;  // ❌ Nested params object
}
```

**After:**
```typescript
interface ChatRequest {
    model: string;
    session_id?: string;
    messages: Message[];
    // Parameters are now flattened (OpenAI-compatible)
    temperature: number;
    top_p: number;
    top_k: number;
    max_tokens: number;
    // Tool support
    tools?: Array<Record<string, any>>;
    tool_choice?: Record<string, any> | string;
    // Advanced features
    reasoning_format?: string;
    reasoning_budget?: number;
    reasoning_budget_message?: string;
    thinking_forced_open?: boolean;
    chat_template_kwargs?: Record<string, unknown>;
    stream: boolean;
}
```

**Why:** The old format didn't match how llama.cpp actually expects requests. The server expects a flat structure following OpenAI's API format.

### 2. Message Interface (Enhanced)

**Before:**
```typescript
interface Message {
    role: Role;
    content: string;
}
```

**After:**
```typescript
interface Message {
    role: string;
    content: string;
    name?: string;              // ✨ New
    tool_call_id?: string;      // ✨ New
    tool_calls?: Array<Record<string, any>>;  // ✨ New
}
```

**Why:** Supports tool calling and matches the Rust `ChatMessage` struct.

### 3. ToolSpec Interface (Breaking Change)

**Before:**
```typescript
interface ToolSpec {
    name: string;
    description: string;
    parameters?: Record<string, unknown>;
}
```

**After:**
```typescript
interface ToolSpec {
    type: 'function';
    function: {
        name: string;
        description: string;
        parameters?: {
            type: 'object';
            properties: Record<string, any>;
            required?: string[];
        };
    };
}
```

**Why:** Matches OpenAI's tool definition format, which llama.cpp expects.

### 4. New Response Types

Added proper response type definitions:

```typescript
interface ChatResponse {
    choices: ChatChoice[];
    usage: Record<string, any>;
}

interface ChatChoice {
    message: Message;
    finish_reason: string;
}

interface IntentClassification {
    needs_external: boolean;
    query: string;
    suggested_tool?: string;
    suggested_server?: string;
    arguments?: Record<string, any>;
    needs_multi_step?: boolean;
    multi_step_reasoning?: string;
}
```

## New Utilities

### ChatRequestBuilder

A fluent builder for constructing chat requests:

```typescript
const request = new ChatRequestBuilder()
    .addSystemMessage('You are helpful')
    .addUserMessage('Hello!')
    .setTemperature(0.8)
    .setMaxTokens(500)
    .addTool(myTool)
    .build();
```

### Helper Functions

```typescript
// Simple request creation
createSimpleChatRequest(
    'What is 2+2?',
    'You are a math tutor',
    { temperature: 0.7 }
);

// Tool creation
createTool(
    'get_weather',
    'Get weather for a location',
    {
        properties: {
            location: { type: 'string' }
        },
        required: ['location']
    }
);
```

## Migration Guide

### If You Were Using the Old ChatRequest

**Old Code:**
```typescript
const request = {
    messages: [{ role: 'user', content: 'Hello' }],
    params: {
        temperature: 0.7,
        max_tokens: 100,
        top_p: 0.9,
        top_k: 40
    }
};
```

**New Code:**
```typescript
const request = {
    model: 'local-model',
    messages: [{ role: 'user', content: 'Hello' }],
    temperature: 0.7,
    max_tokens: 100,
    top_p: 0.9,
    top_k: 40,
    stream: false
};

// Or use the builder:
const request = new ChatRequestBuilder()
    .addUserMessage('Hello')
    .setTemperature(0.7)
    .setMaxTokens(100)
    .build();
```

### If You Were Using Tools

**Old Code:**
```typescript
const request = {
    messages: [...],
    tools: [{
        name: 'get_weather',
        description: 'Get weather',
        parameters: { location: 'string' }
    }],
    params: { ... }
};
```

**New Code:**
```typescript
const request = new ChatRequestBuilder()
    .setMessages(messages)
    .addTool({
        type: 'function',
        function: {
            name: 'get_weather',
            description: 'Get weather',
            parameters: {
                type: 'object',
                properties: {
                    location: { type: 'string' }
                },
                required: ['location']
            }
        }
    })
    .setTemperature(0.7)
    .build();

// Or use the helper:
const tool = createTool('get_weather', 'Get weather', {
    properties: { location: { type: 'string' } },
    required: ['location']
});
```

## Files Changed

### Modified
- `src/lib/types/backend.d.ts` - Updated all chat-related interfaces

### Added
- `src/lib/utils/chat-helpers.ts` - Builder and utility functions
- `src/lib/components/examples/ChatExample.svelte` - Working example
- `docs/CHAT_API.md` - Complete API documentation
- `docs/CHAT_API_MIGRATION.md` - This file

### Updated
- `docs/DOCUMENTATION_INDEX.md` - Added Chat API section

## Verification

The TypeScript types now match:

1. ✅ Rust `ChatRequest` struct in `src-tauri/src/models/chat_model.rs`
2. ✅ llama.cpp's `/v1/chat/completions` endpoint format
3. ✅ OpenAI's chat completions API structure

## Testing

To verify the changes work:

1. Start llama.cpp server with tool support:
   ```bash
   llama-server --model model.gguf --jinja --port 8080
   ```

2. Use the example component:
   ```svelte
   <script>
       import ChatExample from '$lib/components/examples/ChatExample.svelte';
   </script>
   
   <ChatExample />
   ```

3. Or test directly:
   ```typescript
   import { ChatRequestBuilder } from '$lib/utils/chat-helpers';
   
   const request = new ChatRequestBuilder()
       .addUserMessage('Hello!')
       .build();
   
   const response = await fetch('http://localhost:8080/v1/chat/completions', {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify(request)
   });
   ```

## Benefits

1. **Type Safety**: TypeScript types now match actual API contracts
2. **OpenAI Compatibility**: Can use OpenAI documentation as reference
3. **Tool Support**: Proper tool calling with OpenAI format
4. **Better DX**: Builder pattern makes constructing requests easier
5. **Documentation**: Comprehensive docs with examples
6. **Future-Proof**: Aligned with industry standards

## Breaking Changes Summary

⚠️ **Action Required:**

1. Replace `params: GenerationParams` with flattened parameters
2. Update tool definitions to OpenAI format
3. Add `model` and `stream` fields to requests
4. Update any code that constructs `ChatRequest` objects

## Support

- See [CHAT_API.md](./CHAT_API.md) for complete documentation
- Check [ChatExample.svelte](../src/lib/components/examples/ChatExample.svelte) for working code
- Use `ChatRequestBuilder` for type-safe request construction

## Questions?

If you encounter issues:
1. Verify llama.cpp server is running with `--jinja` flag
2. Check that request format matches examples in `CHAT_API.md`
3. Use browser DevTools to inspect actual HTTP requests
4. Compare with Rust implementation in `chat_model.rs`
