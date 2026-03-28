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

## Status & Caching
The backend tracks:
- Connection status.
- Cached tools list.
- Cached resources list.
- Last error (if any).

## Commands
The MCP UI uses Tauri commands:
- `load_mcp_config`, `save_mcp_config`, `reset_mcp_config`, `get_mcp_config_path_string`
- `mcp_add_server`, `mcp_update_server`, `mcp_remove_server`
- `mcp_connect`, `mcp_disconnect`, `mcp_status`
- `mcp_tools_list`, `mcp_tools_call`
- `mcp_resources_list`, `mcp_resources_read`

## Notes
- `transport` values are serialized in camelCase (`httpSse`).
- The **Editar mcp.json** button uses `openPath` via the Tauri opener plugin.
