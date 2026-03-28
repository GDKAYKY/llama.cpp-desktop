# Model Parsing System

This document provides a quick overview of the model parsing implementation.

## What Was Implemented

A complete system for identifying, parsing, and managing Ollama models from the local filesystem.

## Files Created

### Backend (Rust)
- `src-tauri/src/commands/models.rs` - Core model parsing logic with 4 commands:
  - `parse_model_manifest` - Parse a single manifest file
  - `scan_models_directory` - Scan entire models directory
  - `save_model_library` - Save models to JSON
  - `load_model_library` - Load models from JSON

### Frontend (JavaScript/Svelte)
- `src/lib/models.js` - Frontend API wrapper for model commands
- `src/lib/components/ModelSelector.svelte` - UI component for model selection
- `src/routes/models/+page.svelte` - Example page using the component

### Documentation
- `docs/MODELS_SETUP_GUIDE.md` - Complete guide with examples

## Files Modified

- `src-tauri/src/commands/mod.rs` - Added models module
- `src-tauri/src/lib.rs` - Registered model commands

## How It Works

1. **User selects models directory** (e.g., `C:\Users\Name\.ollama\models`)
2. **App scans manifests folder** following structure: `manifests/{provider}/{library}/{name}/{version}`
3. **Parses each manifest file** extracting metadata and layer information
4. **Maps digests to blob files** converting `sha256:abc...` to `sha256-abc...`
5. **Saves to modelLibrary.json** for persistent storage

## Quick Start

### Use the UI Component
```svelte
<script>
  import ModelSelector from '$lib/components/ModelSelector.svelte';
</script>

<ModelSelector />
```

### Use the API Directly
```javascript
import { scanModelsDirectory, saveModelLibrary } from '$lib/models.js';

const models = await scanModelsDirectory('C:/Users/Name/.ollama/models');
await saveModelLibrary('C:/Users/Name/.ollama/models/modelLibrary.json', models);
```

## Model Identifier Format

Models are identified as: `{provider}:{name}:{version}`

Example: `registry.ollama.ai:qwen2.5-coder:7b`

## Data Structure

Each model object contains:
```javascript
{
  provider: "registry.ollama.ai",
  library: "library",
  name: "qwen2.5-coder",
  version: "7b",
  full_identifier: "registry.ollama.ai:qwen2.5-coder:7b",
  model_file_path: "C:/.../.ollama/models/blobs/sha256-60e05f21...",
  manifest: { /* complete manifest data */ }
}
```

## Testing

To test the implementation:

1. Navigate to `/models` route in the app
2. Click "Select Models Directory"
3. Choose your Ollama models folder (usually `~/.ollama/models`)
4. Click "Scan for Models"
5. View the parsed models with their metadata

## Next Steps

Consider adding:
- Model filtering and search
- Model metadata editing
- Model download/import functionality
- Integration with chat interface for model selection
- Model performance metrics
