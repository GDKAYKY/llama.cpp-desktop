# Models Setup Guide

## Overview

This guide explains how to configure and use the model library system in Llama Desktop.

## Directory Structure

Models should be organized in the following structure:

```
library/
├── model-name-1/
│   ├── version-1/
│   │   └── manifest          (JSON file without extension)
│   └── version-2/
│       └── manifest
├── model-name-2/
│   └── version-1/
│       └── manifest
```

## Manifest File Format

The `manifest` file (without `.json` extension) should contain:

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "digest": "sha256:...",
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "size": 1234
  },
  "layers": [
    {
      "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
      "digest": "sha256:...",
      "size": 5678
    }
  ]
}
```

## Frontend Usage

### 1. Configure Library Path

When the app starts, use the **Path Selector** component to set the path to your models library:

```
Configure Model Library Path
[Enter path to models library] [Set Path]
```

Example paths:
- Windows: `C:/Users/username/models` or `E:/models`
- Linux/Mac: `/home/username/models` or `/Users/username/models`

### 2. Load Models

After setting the path, the **Model Selector** will automatically load available models:

```
Available Models
[Select a model...] [Load Model] [Unload]
```

Select a model and click "Load Model" to use it.

### 3. Use Chat Interface

Once a model is loaded, the **Chat Interface** becomes available for interaction.

## Backend API Endpoints

### Set Library Path

```http
POST /api/models/library-path
Content-Type: application/json

{
  "libraryPath": "C:/models"
}
```

Response:
```json
{
  "success": true,
  "data": {
    "message": "Library path configured successfully",
    "libraryPath": "C:/models"
  }
}
```

### List Models

```http
GET /api/models
```

Response:
```json
{
  "success": true,
  "data": {
    "libraryPath": "C:/models",
    "models": [
      {
        "modelName": "llama",
        "version": "2.0",
        "path": "C:/models/library/llama/2.0",
        "manifest": {
          "schemaVersion": 2,
          "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
          "config": {...},
          "layers": [...]
        }
      }
    ],
    "total": 1
  }
}
```

### Select Model

```http
POST /api/models/select
Content-Type: application/json

{
  "modelName": "llama",
  "version": "2.0"
}
```

Response:
```json
{
  "success": true,
  "data": {
    "modelName": "llama",
    "version": "2.0",
    "path": "C:/models/library/llama/2.0",
    "manifest": {...},
    "message": "Model llama:2.0 selected successfully"
  }
}
```

### Get Selected Model

```http
GET /api/models/selected
```

Response:
```json
{
  "success": true,
  "data": {
    "modelName": "llama",
    "version": "2.0",
    "path": "C:/models/library/llama/2.0",
    "manifest": {...}
  }
}
```

## Caching

Models are cached in memory after the first load. The cache is cleared when:
- A new library path is set
- The application restarts

This ensures optimal performance when listing and selecting models.

## Error Handling

### Path Not Found

```json
{
  "success": false,
  "error": {
    "code": "PATH_NOT_FOUND",
    "message": "Path does not exist: C:/invalid/path"
  }
}
```

### Model Not Found

```json
{
  "success": false,
  "error": {
    "code": "MODEL_NOT_FOUND",
    "message": "Model llama:3.0 not found"
  }
}
```

### Invalid Path

```json
{
  "success": false,
  "error": {
    "code": "NOT_A_DIRECTORY",
    "message": "Path is not a directory: C:/models/file.txt"
  }
}
```

## Troubleshooting

### Models not appearing

1. Verify the library path is correct
2. Check that the directory structure matches the expected format
3. Ensure manifest files exist (without `.json` extension)
4. Check the backend logs for errors

### Path configuration fails

1. Verify the path exists on your system
2. Check file permissions
3. Ensure the path is a directory, not a file

### Model loading fails

1. Verify the manifest file is valid JSON
2. Check that all required fields are present
3. Ensure the model path is accessible
