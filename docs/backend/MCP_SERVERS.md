# MCP Servers

This document describes how MCP servers are configured, managed, and used in Llama Desktop.

## Overview

MCP servers expose tools and resources that can be connected from the app. Configuration is stored locally and managed through the MCP UI.

## Configuration File

- **Path**: `{app_config_dir}/mcp.json`
- The app will create this file if it does not exist.

## Schema

The config is a JSON object with a `servers` array.

```json
{
  "servers": [
    {
      "id": "my-server",
      "name": "My MCP Server",
      "enabled": true,
      "transport": "stdio",
      "command": "mcp-server",
      "args": ["--flag", "value"],
      "cwd": "/path/to/dir",
      "env": { "KEY": "VALUE" },
      "url": null,
      "headers": null,
      "tool_allowlist": ["toolA", "toolB"],
      "resource_allowlist": ["file:///path", "mcp://resource"]
    }
  ]
}
```

### Fields

- `id` (string): Unique identifier for the server.
- `name` (string): Display name in the UI.
- `enabled` (boolean): Whether the server can be connected.
- `transport` (enum): `stdio` or `httpSse`.
- `command` (string, stdio): Executable to launch.
- `args` (string[], stdio): Arguments for the executable.
- `cwd` (string, stdio): Working directory.
- `env` (object, stdio): Environment variables.
- `url` (string, httpSse): Base URL of the MCP server.
- `headers` (object, httpSse): Optional HTTP headers.
- `tool_allowlist` (string[], optional): Restrict available tools by name.
- `resource_allowlist` (string[], optional): Restrict accessible resource URIs.

## UI Workflow

1. Open **MCP Servers**.
2. Add or edit a server.
3. Click **Connect** to start the session.
4. List tools/resources to verify connectivity.

## Local npm install (no npx)

You can install MCP servers locally and run them from `node_modules/.bin`.

### Install

```bash
npm install @modelcontextprotocol/server-filesystem
```

### Example config

```json
{
  "servers": [
    {
      "id": "filesystem",
      "name": "Filesystem MCP (Local)",
      "enabled": true,
      "transport": "stdio",
      "command": "node_modules/.bin/mcp-server-filesystem",
      "args": ["--allow", "C:/"],
      "cwd": "<repo-root>",
      "env": null,
      "url": null,
      "headers": null,
      "tool_allowlist": null,
      "resource_allowlist": null
    }
  ]
}
```

### Windows note

On Windows, the local bin is `node_modules/.bin/mcp-server-filesystem.cmd`.

## Status & Caching

The backend tracks:

- Connection status.
- Cached tools list.
- Cached resources list.
- Last error (if any).

## Embedded Node (npx)

The app does **not** ship Node by default. If your MCP server command uses `npx`, you must either:

- Have Node installed and available in the PATH of the GUI process, or
- Embed Node binaries in the app bundle.

When Node is embedded, the MCP resolver looks in the bundled resources before failing:

- `resources/node/bin/npx.cmd` and `resources/node/bin/node.exe` (Windows)
- `resources/node/bin/npx` and `resources/node/bin/node` (macOS/Linux)

### Bundle Setup

1. Place Node/NPM binaries in `src-tauri/resources/node/bin/` as shown above.
2. Ensure `tauri.conf.json` includes:
   - `bundle.resources: ["resources/node/**"]`
3. Rebuild the Tauri app so the resources are packaged.

## Commands

The MCP UI uses Tauri commands:

- `load_mcp_config`, `save_mcp_config`, `reset_mcp_config`, `get_mcp_config_path_string`
- `add_server`, `update_server`, `remove_server`
- `connect`, `disconnect`, `status`
- `list_tools`, `call_tools`
- `list_resources`, `read_resources`

## Notes

- `transport` values are serialized in camelCase (`httpSse`).
- The **Editar mcp.json** button uses `openPath` via the Tauri opener plugin.
- If you see "program not found" when connecting via `npx`, it means Node is not available to the GUI process and must be installed or embedded.
