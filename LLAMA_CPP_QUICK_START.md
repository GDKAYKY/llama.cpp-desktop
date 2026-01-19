# llama.cpp Integration - Quick Start

## What's Been Added

Your Tauri application now has full llama.cpp integration with:

1. **Rust Backend** (`src-tauri/src/services/llama_cpp.rs`)
   - Manages llama-server process lifecycle
   - Handles HTTP requests to the local server
   - Supports chat completions with configurable parameters

2. **Tauri Commands** (`src-tauri/src/commands/llama_cpp.rs`)
   - `start_llama_server` - Start the server with a model
   - `stop_llama_server` - Stop the running server
   - `is_llama_running` - Check server status
   - `get_llama_config` - Get current configuration
   - `send_chat_message` - Send a single message
   - `send_chat_with_history` - Send message with conversation history

3. **Svelte Component** (`src/lib/components/LlamaCppChat.svelte`)
   - Complete chat UI with server management
   - Configuration panel for server settings
   - Chat parameters adjustment (temperature, top_p, etc.)
   - Message history display

## Quick Setup

### 1. Prepare Your Environment

```
E:\src\llama_cpp\          # Your llama.cpp folder with executables
C:\Models\mistral-7b.gguf  # Your GGUF model file
```

### 2. Use the Component

In your Svelte page:

```svelte
<script>
  import LlamaCppChat from '$lib/components/LlamaCppChat.svelte';
</script>

<LlamaCppChat />
```

### 3. Or Use Commands Directly

```javascript
import { invoke } from '@tauri-apps/api/core';

// Start server
await invoke('start_llama_server', {
  llama_cpp_path: 'E:\\src\\llama_cpp',
  model_path: 'C:\\Models\\mistral-7b.gguf',
  port: 8080,
  ctx_size: 4096,
  parallel: 4,
  n_gpu_layers: 33,
});

// Send message
const response = await invoke('send_chat_message', {
  message: 'Hello!',
  temperature: 0.7,
  top_p: 0.95,
  top_k: 40,
  max_tokens: 512,
});

// Stop server
await invoke('stop_llama_server');
```

## Key Parameters

| Parameter        | Purpose                    | Example                     |
| ---------------- | -------------------------- | --------------------------- |
| `llama_cpp_path` | Path to llama.cpp folder   | `E:\src\llama_cpp`          |
| `model_path`     | Path to GGUF model         | `C:\Models\mistral-7b.gguf` |
| `port`           | HTTP server port           | `8080`                      |
| `ctx_size`       | Context window (tokens)    | `4096`                      |
| `parallel`       | Concurrent inference slots | `4`                         |
| `n_gpu_layers`   | GPU layers to offload      | `33` (or 0 for CPU)         |
| `temperature`    | Randomness (0-2)           | `0.7`                       |
| `top_p`          | Nucleus sampling           | `0.95`                      |
| `top_k`          | Top-k sampling             | `40`                        |
| `max_tokens`     | Max response length        | `512`                       |

## Performance Tips

- **GPU**: Set `n_gpu_layers` high (e.g., 33) for faster inference
- **Memory**: Reduce `ctx_size` or `parallel` if running out of memory
- **Speed**: Use smaller models (7B) or higher quantization (Q4)
- **Quality**: Increase `ctx_size` and use lower quantization (Q8) for better quality

## Troubleshooting

**Server won't start**
- Check paths are correct and files exist
- Ensure port 8080 is not in use
- Verify llama-server.exe is in the llama_cpp folder

**Slow responses**
- Reduce context size or parallel slots
- Use a smaller model
- Enable GPU acceleration

**Out of memory**
- Reduce `ctx_size` (try 2048)
- Reduce `parallel` (try 1-2)
- Use a smaller model or higher quantization

## Files Modified/Created

```
src-tauri/
├── Cargo.toml (added dependencies)
├── src/
│   ├── lib.rs (registered commands)
│   ├── commands/
│   │   ├── llama_cpp.rs (NEW)
│   │   └── mod.rs (updated)
│   ├── services/
│   │   ├── llama_cpp.rs (NEW)
│   │   └── mod.rs (updated)
│   └── state/
│       └── mod.rs (updated)

src/
└── lib/components/
    └── LlamaCppChat.svelte (NEW)

docs/
└── LLAMA_CPP_INTEGRATION.md (NEW - detailed guide)
```

## Next Steps

1. Update the default paths in `LlamaCppChat.svelte` to match your setup
2. Test with a small model first (e.g., 7B)
3. Adjust parameters based on your hardware
4. Integrate the chat component into your app

## Documentation

See `docs/LLAMA_CPP_INTEGRATION.md` for detailed API documentation and examples.
