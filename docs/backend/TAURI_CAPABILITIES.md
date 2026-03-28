# Tauri v2 Capabilities Guide

This guide explains the Tauri v2 security configuration and capabilities used in Llama Desktop, specifically within `src-tauri/capabilities/default.json`.

## Core Configuration

The `default.json` file controls what the frontend (JavaScript/TypeScript) is allowed to perform via IPC.

### File System Permissions

The application requires specific file system permissions to manage models and app data.

- **`fs:allow-mkdir`**: This is the correct permission for creating directories in Tauri v2 (Replaced the non-existent `fs:allow-create-dir`).
- **`fs:default`**: Provides safe default read access to app-specific folders.
- **`fs:allow-read`, `fs:allow-write`, etc.**: Enables core file system operations.

### File System Scope (Path Restrictions)

In Tauri v2, enabling commands is not sufficient. You must also define which folders the frontend is authorized to access using the `fs.scope` property.

#### Recommended Scopes

- **`$APPDATA/**`**: persistent app data, models, settings, and chat history.
- **`$APPLOCALDATA/**`**: local (non-roaming) data, ideal for large model files and caches.
- **`$APPCONFIG/**`**: application configuration files.

#### Reference Implementation

Below is a reference of how the `permissions` and `fs` scope should be configured:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window - controls what the frontend (JavaScript/TypeScript) is allowed to do",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "shell:default",
    "shell:allow-open",
    "dialog:default",

    // === File System Permissions (commands) ===
    "fs:default",           // Safe default read access to app-specific folders
    "fs:allow-read",
    "fs:allow-write",
    "fs:allow-exists",
    "fs:allow-lstat",
    "fs:allow-stat",
    "fs:allow-mkdir",       // Correct permission for creating directories
    "fs:allow-remove",
    "fs:allow-rename",
    "fs:allow-copy-file",

    // Core Tauri modules
    { "identifier": "core:app:default" },
    { "identifier": "core:event:default" },
    { "identifier": "core:path:default" },
    { "identifier": "core:window:default" },
    { "identifier": "core:menu:default" },
    { "identifier": "core:tray:default" }
  ],

  // === IMPORTANT: FS Scope (Path Restrictions) ===
  "fs": {
    "scope": [
      "$APPDATA/**",       // App data, models, settings, chats
      "$APPDATA",          // Base folder
      "$APPLOCALDATA/**",  // Local caches, large model files
      "$APPLOCALDATA",
      "$APPCONFIG/**",     // Configuration files
      "$APPCONFIG"
    ]
  }
}
```

## Troubleshooting

If you encounter "forbidden path" errors at runtime despite having the permissions enabled, check that the target path is explicitly listed in the `fs.scope` array.

After making changes to the capability configuration, it is recommended to run:
1. `cargo clean`
2. `npm run tauri dev`
