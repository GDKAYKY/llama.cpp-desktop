# llama.cpp Integration Guide

This document explains how to use the integrated llama.cpp backend in your Tauri application.

## Setup

### 1. Configure llama.cpp Path

Users need to have llama.cpp executables in a folder. The expected structure is:

```
E:\src\llama_cpp\
├── llama-server.exe
├── llama-cli.exe
├── llama-tokenize.exe
├── ggml-base.dll
├── ggml-cuda.dll
├── llama.dll
└── ... (other DLLs and executables)
```

### 2. Prepare GGUF Models

Place your GGUF format models in a known location. Example:

```
C:\Models\
├── mistral-7b.gguf
├── llama2-13b.gguf
└── gemma-7b.gguf
```

## Frontend Usage (JavaScript/TypeScript)

### Starting the Server

```javascript
import { invoke } from '@tauri-apps/api/core';

async function startLlamaServer() {
  try {
    const result = await invoke('start_llama_server', {
      llama_cpp_path: 'E:\\src\\llama_cpp',
      model_path: 'C:\\Models\\mistral-7b.gguf',
      port: 8080,
      ctx_size: 4096,
      parallel: 4,
      n_gpu_layers: 33, // Set to 0 for CPU only, or higher for GPU acceleration
    });
    console.log(result); // "llama-server started successfully"
  } catch (error) {
    console.error('Failed to start server:', error);
  }
}
```

### Checking Server Status

```javascript
async function checkServerStatus() {
  const isRunning = await invoke('is_llama_running');
  console.log('Server running:', isRunning);
}
```

### Sending a Chat Message

```javascript
import { Channel } from '@tauri-apps/api/core';

async function sendMessage(userMessage, sessionId) {
  const onEvent = new Channel();
  onEvent.onmessage = (payload) => {
    if (payload.chunk) {
      console.log('Chunk:', payload.chunk);
    }
    if (payload.status === 'done') {
      console.log('Finished');
    }
  };

  try {
    await invoke('send_message', {
      message: userMessage,
      sessionId: sessionId || crypto.randomUUID(),
      temperature: 0.7,
      maxTokens: 512,
      onEvent
    });
  } catch (error) {
    console.error('Failed to send message:', error);
  }
}
```

### Sending Chat with History

The current implementation uses `session_id` to manage conversation history server-side. Simply continue sending messages with the same `sessionId`.

```javascript
// Already handled by send_message when using the same sessionId
```

### Getting Current Configuration

```javascript
async function getServerConfig() {
  const config = await invoke('get_llama_config');
  console.log('Current config:', config);
  // Returns: { llama_cpp_path, model_path, port, ctx_size, parallel, n_gpu_layers }
}
```

### Stopping the Server

```javascript
async function stopServer() {
  try {
    const result = await invoke('stop_llama_server');
    console.log(result); // "llama-server stopped"
  } catch (error) {
    console.error('Failed to stop server:', error);
  }
}
```

## Configuration Parameters

### start_llama_server Parameters

| Parameter        | Type   | Description                                                     |
| ---------------- | ------ | --------------------------------------------------------------- |
| `llama_cpp_path` | string | Path to llama.cpp folder containing executables                 |
| `model_path`     | string | Full path to the GGUF model file                                |
| `port`           | u16    | HTTP server port (default: 8080)                                |
| `ctx_size`       | u32    | Context window size in tokens (default: 4096)                   |
| `parallel`       | u32    | Number of parallel inference slots (default: 4)                 |
| `n_gpu_layers`   | i32    | Number of layers to offload to GPU (-1 for all, 0 for CPU only) |

### Chat Parameters

| Parameter     | Type | Description                                          |
| ------------- | ---- | ---------------------------------------------------- |
| `temperature` | f32  | Sampling temperature (0.0-2.0, higher = more random) |
| `top_p`       | f32  | Nucleus sampling parameter (0.0-1.0)                 |
| `top_k`       | i32  | Top-k sampling (0 = disabled)                        |
| `max_tokens`  | i32  | Maximum tokens to generate                           |

## Example: Complete Chat Application

```javascript
let conversationHistory = [];

async function initializeApp() {
  // Start server
  await startLlamaServer();
  
  // Check status
  const running = await invoke('is_llama_running');
  console.log('Server ready:', running);
}

async function handleUserInput(userMessage) {
  // Add user message to history
  conversationHistory.push(['user', userMessage]);
  
  // Send with history
  const response = await sendChatWithHistory(conversationHistory);
  
  // Add assistant response to history
  conversationHistory.push(['assistant', response]);
  
  return response;
}

async function cleanup() {
  await stopServer();
}
```

## Performance Tips

1. **GPU Acceleration**: Set `n_gpu_layers` to a high value (e.g., 33 for 7B models) to offload computation to GPU
2. **Context Size**: Larger context sizes use more memory. Start with 2048 or 4096
3. **Parallel Slots**: More parallel slots allow concurrent requests but use more memory
4. **Model Selection**: Smaller quantized models (Q4, Q5) are faster than larger ones (Q8)

## Troubleshooting

### Server fails to start
- Verify llama.cpp path exists and contains `llama-server.exe`
- Check model file path is correct and file exists
- Ensure port is not already in use

### Slow responses
- Reduce `ctx_size` or `parallel` to free up memory
- Use a smaller model or higher quantization (Q4 instead of Q8)
- Enable GPU acceleration with `n_gpu_layers`

### Out of memory errors
- Reduce `ctx_size`
- Reduce `parallel` (fewer concurrent requests)
- Use a smaller model
- Reduce `max_tokens` in chat requests

## API Endpoints (Direct HTTP)

If you need to call the server directly via HTTP:

```bash
# Chat completion
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer no-key" \
  -d '{
    "model": "llama",
    "session_id": "user-123",
    "messages": [
      {"role": "user", "content": "Hello"}
    ],
    "temperature": 0.7
  }'

# Get server slots status
curl http://localhost:8080/slots

# Get metrics
curl http://localhost:8080/metrics
```

## References

- [llama.cpp GitHub](https://github.com/ggerganov/llama.cpp)
- [GGUF Format](https://github.com/ggerganov/ggml/blob/master/docs/gguf.md)
- [Hugging Face GGUF Models](https://huggingface.co/models?search=gguf)
