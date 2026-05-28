# Chat API Documentation

## Overview

Chat generation now runs in the frontend through Vercel AI SDK using llama.cpp's OpenAI-compatible `/v1/chat/completions` endpoint. Rust is infrastructure only for process lifecycle, raw llama.cpp HTTP proxying, and MCP server IPC; it no longer owns conversation session state or the MCP tool loop.

## Type Definitions

### ChatRequest

The main request interface for chat completions:

```typescript
interface ChatRequest {
    model: string;                              // Model identifier
    session_id?: string;                        // Optional session for context persistence
    messages: Message[];                        // Conversation history
    temperature: number;                        // Randomness (0.0-2.0)
    top_p: number;                             // Nucleus sampling (0.0-1.0)
    top_k: number;                             // Top-k sampling
    max_tokens: number;                        // Maximum tokens to generate
    reasoning_format?: string;                 // Enable reasoning mode (e.g., "thinking")
    reasoning_budget?: number;                 // Token budget for reasoning
    reasoning_budget_message?: string;         // Custom budget message
    thinking_forced_open?: boolean;            // Force thinking tags open
    chat_template_kwargs?: Record<string, unknown>; // Template customization
    tools?: Array<Record<string, any>>;        // OpenAI-format tool definitions
    tool_choice?: Record<string, any> | string; // Tool selection strategy
    stream: boolean;                           // Enable streaming responses
}
```

### Message

OpenAI-compatible message format:

```typescript
interface Message {
    role: string;                              // 'user', 'assistant', 'system', 'tool'
    content: string;                           // Message content
    name?: string;                             // Optional name for the message
    tool_call_id?: string;                     // ID for tool response messages
    tool_calls?: Array<Record<string, any>>;   // Tool calls made by assistant
}
```

### ToolSpec

OpenAI-compatible tool definition:

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

## Usage Examples

### Basic Chat Request

```typescript
import { ChatRequestBuilder } from '$lib/utils/chat-helpers';

const request = new ChatRequestBuilder()
    .addSystemMessage('You are a helpful assistant.')
    .addUserMessage('What is the capital of France?')
    .setTemperature(0.7)
    .setMaxTokens(100)
    .build();

// Send to llama.cpp server
const response = await fetch('http://localhost:8080/v1/chat/completions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request)
});
```

### Using the Simple Helper

```typescript
import { createSimpleChatRequest } from '$lib/utils/chat-helpers';

const request = createSimpleChatRequest(
    'Explain quantum computing',
    'You are a physics teacher.',
    { temperature: 0.8, max_tokens: 500 }
);
```

### Chat with Tools

```typescript
import { ChatRequestBuilder, createTool } from '$lib/utils/chat-helpers';

// Define a tool
const weatherTool = createTool(
    'get_weather',
    'Get current weather for a location',
    {
        properties: {
            location: { type: 'string', description: 'City name' },
            units: { 
                type: 'string', 
                enum: ['celsius', 'fahrenheit'],
                description: 'Temperature units'
            }
        },
        required: ['location']
    }
);

// Build request with tool
const request = new ChatRequestBuilder()
    .addUserMessage('What is the weather in Paris?')
    .addTool(weatherTool)
    .setToolChoice('auto')  // Let model decide when to use tools
    .build();
```

### Multi-turn Conversation

```typescript
const builder = new ChatRequestBuilder()
    .addSystemMessage('You are a helpful coding assistant.')
    .addUserMessage('How do I reverse a string in Python?')
    .addAssistantMessage('You can use slicing: `text[::-1]`')
    .addUserMessage('Can you show a full example?')
    .build();
```

### Streaming Responses

```typescript
const request = new ChatRequestBuilder()
    .addUserMessage('Write a short story')
    .setStream(true)
    .setMaxTokens(1000)
    .build();

const response = await fetch('http://localhost:8080/v1/chat/completions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request)
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    const lines = chunk.split('\n').filter(line => line.trim());
    
    for (const line of lines) {
        if (line.startsWith('data: ')) {
            const data = line.slice(6);
            if (data === '[DONE]') continue;
            
            const parsed = JSON.parse(data);
            const content = parsed.choices[0]?.delta?.content;
            if (content) {
                console.log(content);
            }
        }
    }
}
```

### Reasoning/Thinking Mode

For models that support extended reasoning (like DeepSeek R1):

```typescript
const request = new ChatRequestBuilder()
    .addUserMessage('Solve this logic puzzle: ...')
    .enableReasoning(2000, 'thinking')  // 2000 token budget
    .setMaxTokens(4000)
    .build();
```

## Response Format

### ChatResponse

```typescript
interface ChatResponse {
    choices: ChatChoice[];
    usage: Record<string, any>;
}

interface ChatChoice {
    message: Message;
    finish_reason: string;  // 'stop', 'length', 'tool_calls', etc.
}
```

### Example Response

```json
{
    "choices": [{
        "message": {
            "role": "assistant",
            "content": "The capital of France is Paris."
        },
        "finish_reason": "stop"
    }],
    "usage": {
        "prompt_tokens": 15,
        "completion_tokens": 8,
        "total_tokens": 23
    }
}
```

### Tool Call Response

```json
{
    "choices": [{
        "message": {
            "role": "assistant",
            "content": "",
            "tool_calls": [{
                "id": "call_123",
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "arguments": "{\"location\": \"Paris\", \"units\": \"celsius\"}"
                }
            }]
        },
        "finish_reason": "tool_calls"
    }],
    "usage": { ... }
}
```

## Migration from Old Format

### Before (Incorrect)

```typescript
// Old format with nested params
const request = {
    messages: [{ role: 'user', content: 'Hello' }],
    tools: [{ name: 'tool', description: 'desc' }],  // Wrong format
    params: {
        temperature: 0.7,
        max_tokens: 100,
        top_p: 0.9,
        top_k: 40
    }
};
```

### After (Correct)

```typescript
// New OpenAI-compatible format
const request = {
    model: 'local-model',
    messages: [{ role: 'user', content: 'Hello' }],
    tools: [{  // OpenAI format
        type: 'function',
        function: {
            name: 'tool',
            description: 'desc',
            parameters: { type: 'object', properties: {} }
        }
    }],
    temperature: 0.7,      // Flattened
    max_tokens: 100,
    top_p: 0.9,
    top_k: 40,
    stream: false
};
```

## Internal API (Tauri Commands)

The orchestrator service provides a simplified internal API:

```typescript
import { OrchestratorService } from '$lib/services/orchestrator';

// Create a conversation slot
const slotId = await OrchestratorService.createSlot(10);

// Send a message (simplified interface)
const response = await OrchestratorService.sendMessage(
    slotId,
    'Hello!',
    { temperature: 0.8, max_tokens: 200 }  // Optional params
);

// Get conversation history
const messages = await OrchestratorService.getSlotMessages(slotId);
```

This internal API automatically handles:
- Session management
- Message history
- Default parameters
- Conversion to full ChatRequest format

## Best Practices

1. **Use the Builder Pattern**: `ChatRequestBuilder` provides type safety and validation
2. **Set Reasonable Defaults**: Use `DEFAULT_CHAT_CONFIG` as a starting point
3. **Handle Streaming Properly**: Always check for `[DONE]` marker in SSE streams
4. **Validate Tool Definitions**: Ensure `parameters.type` is always `'object'`
5. **Session Management**: Use `session_id` for multi-turn conversations with context
6. **Error Handling**: Check `finish_reason` for `'length'` (truncated) or `'error'`

## llama.cpp Server Configuration

For tool calling to work properly, start llama.cpp server with:

```bash
llama-server \
    --model model.gguf \
    --ctx-size 8192 \
    --parallel 4 \
    --jinja \
    --chat-template-file template.jinja
```

Key flags:
- `--jinja`: Enable Jinja2 template support (required for tools)
- `--chat-template-file`: Custom chat template with tool support
- `--ctx-size`: Context window size
- `--parallel`: Number of parallel requests

## References

- [llama.cpp Server API](https://github.com/ggerganov/llama.cpp/blob/master/examples/server/README.md)
- [OpenAI Chat Completions API](https://platform.openai.com/docs/api-reference/chat)
- [Tool Calling Guide](https://platform.openai.com/docs/guides/function-calling)
