# Models Setup Guide

This guide explains how the Llama Desktop application identifies, parses, and manages models from your local Ollama installation.

## Overview

The application can automatically scan and parse model manifests from your Ollama models directory, extracting metadata and creating a searchable model library.

## Model Directory Structure

Models are stored in the following structure:
```
{modelsRoot}/
├── manifests/
│   └── {modelProvider}/
│       └── {manifestLibrary}/
│           └── {modelName}/
│               └── {modelVersion}  (manifest file)
└── blobs/
    └── sha256-{hash}  (actual model files)
```

Example:
```
C:/Users/YourName/.ollama/models/
├── manifests/
│   └── registry.ollama.ai/
│       └── library/
│           └── qwen2.5-coder/
│               └── 7b  (manifest file)
└── blobs/
    └── sha256-60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463
```

## Manifest File Format

Each model version file is a JSON manifest containing:

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "digest": "sha256:d9bb33f2786931fea42f50936a2424818aa2f14500638af2f01861eb2c8fb446",
    "size": 487
  },
  "layers": [
    {
      "mediaType": "application/vnd.ollama.image.model",
      "digest": "sha256:60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463",
      "size": 4683074048
    },
    {
      "mediaType": "application/vnd.ollama.image.system",
      "digest": "sha256:66b9ea09bd5b7099cbb4fc820f31b575c0366fa439b08245566692c6784e281e",
      "size": 68
    },
    {
      "mediaType": "application/vnd.ollama.image.template",
      "digest": "sha256:1e65450c30670713aa47fe23e8b9662bdf4065e81cc8e3cbfaa98924fcc0d320",
      "size": 1615
    },
    {
      "mediaType": "application/vnd.ollama.image.license",
      "digest": "sha256:832dd9e00a68dd83b3c3fb9f5588dad7dcf337a0db50f7d9483f310cd292e92e",
      "size": 11343
    }
  ]
}
```

## How It Works

### 1. Model Identification

When you select a models directory, the app:
- Scans the `manifests/` folder structure
- Extracts provider, library, name, and version from the path
- Creates a full identifier: `{provider}:{name}:{version}`

Example: `registry.ollama.ai:qwen2.5-coder:7b`

### 2. Manifest Parsing

For each manifest file, the app:
- Parses the JSON structure
- Extracts layer information (model file, system prompts, templates, license)
- Identifies the main model layer (mediaType: `application/vnd.ollama.image.model`)

### 3. Blob File Mapping

The digest from the manifest is converted to a blob filename:
- Digest: `sha256:60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463`
- Blob file: `sha256-60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463`

The app verifies the blob file exists in the `blobs/` directory.

### 4. Model Library Storage

All parsed models are saved to `modelLibrary.json`:

```json
{
  "models": [
    {
      "provider": "registry.ollama.ai",
      "library": "library",
      "name": "qwen2.5-coder",
      "version": "7b",
      "full_identifier": "registry.ollama.ai:qwen2.5-coder:7b",
      "model_file_path": "C:/Users/YourName/.ollama/models/blobs/sha256-60e05f21...",
      "manifest": { /* full manifest data */ }
    }
  ]
}
```

## Usage

### Frontend API

```javascript
import { 
  selectModelsDirectory,
  scanModelsDirectory,
  saveModelLibrary,
  loadModelLibrary 
} from '$lib/models.js';

// Let user select models directory
const modelsRoot = await selectModelsDirectory();

// Scan for all models
const models = await scanModelsDirectory(modelsRoot);

// Save to library
await saveModelLibrary(`${modelsRoot}/modelLibrary.json`, models);

// Load existing library
const existingModels = await loadModelLibrary(`${modelsRoot}/modelLibrary.json`);
```

### Using the ModelSelector Component

```svelte
<script>
  import ModelSelector from '$lib/components/ModelSelector.svelte';
</script>

<ModelSelector />
```

The component provides:
- Directory selection dialog
- Automatic scanning and parsing
- Visual model cards with metadata
- Model selection interface
- Persistent storage in modelLibrary.json

## Backend Commands

The Rust backend provides these commands:

### `parse_model_manifest`
Parse a single manifest file.

```javascript
await invoke('parse_model_manifest', {
  modelPath: 'path/to/manifest/file',
  modelsRoot: 'path/to/models/root'
});
```

### `scan_models_directory`
Scan entire models directory.

```javascript
await invoke('scan_models_directory', {
  modelsRoot: 'path/to/models/root'
});
```

### `save_model_library`
Save models to JSON file.

```javascript
await invoke('save_model_library', {
  libraryPath: 'path/to/modelLibrary.json',
  models: [/* array of model objects */]
});
```

### `load_model_library`
Load models from JSON file.

```javascript
await invoke('load_model_library', {
  libraryPath: 'path/to/modelLibrary.json'
});
```

## Model Information Structure

Each parsed model contains:

- `provider`: Model provider (e.g., "registry.ollama.ai")
- `library`: Manifest library (e.g., "library")
- `name`: Model name (e.g., "qwen2.5-coder")
- `version`: Model version (e.g., "7b")
- `full_identifier`: Complete identifier string
- `model_file_path`: Path to the actual model blob file (if found)
- `manifest`: Complete manifest data including:
  - `schema_version`: Manifest schema version
  - `media_type`: Manifest media type
  - `config`: Configuration metadata
  - `layers`: Array of layers (model, system, template, license)

## Common Locations

### Windows
```
C:\Users\{Username}\.ollama\models
```

### macOS
```
~/.ollama/models
```

### Linux
```
~/.ollama/models
```

## Troubleshooting

### Model file not found
- Ensure the blobs directory exists in your models root
- Verify the digest in the manifest matches a blob file
- Check file permissions

### No models found
- Verify the manifests directory structure is correct
- Ensure manifest files are valid JSON
- Check that you selected the correct root directory (should contain both `manifests/` and `blobs/`)

### Invalid path structure
- The path must follow: `.../manifests/{provider}/{library}/{name}/{version}`
- Ensure you're selecting the models root, not a subdirectory
