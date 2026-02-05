# Configuration System Guide

## Overview

The Llama Desktop application includes a comprehensive configuration system that allows users to customize their experience through a settings page.

## Features

### Settings Categories

1. **Models**
   - Models Directory: Select where your Ollama models are stored
   - Automatically integrates with the model scanning system

2. **Appearance**
   - Theme: Dark, Light, or Auto (follows system)
   - Language: Interface language selection (EN, ES, FR, DE, ZH)

3. **Model Parameters**
   - Max Tokens: Control generation length (128-8192)
   - Temperature: Adjust creativity vs focus (0.0-2.0)

4. **Chat Settings**
   - Auto-save Chat History: Toggle automatic conversation saving
   - Chat History Limit: Number of recent chats to keep (10-100)

5. **About**
   - Version information
   - Configuration file location

## Usage

### Accessing Settings

Navigate to `/settings` or click the Settings link in the sidebar.

### Saving Changes

1. Modify any settings
2. Click "Save Changes" button
3. Changes are persisted to disk immediately

### Resetting to Defaults

Click "Reset to Defaults" to restore all settings to their original values.

## Configuration File

Settings are stored in a JSON file at:
- **Windows**: `%APPDATA%\com.llama-desktop.app\config.json`
- **macOS**: `~/Library/Application Support/com.llama-desktop.app/config.json`
- **Linux**: `~/.config/com.llama-desktop.app/config.json`

### Configuration Structure

```json
{
  "models_directory": "C:/Users/Name/.ollama/models",
  "theme": "dark",
  "language": "en",
  "max_tokens": 2048,
  "temperature": 0.7,
  "auto_save_chat": true,
  "chat_history_limit": 50
}
```

## API Reference

### Frontend (JavaScript)

```javascript
import { loadConfig, saveConfig, resetConfig, getConfigPath } from '$lib/config.js';

// Load configuration
const config = await loadConfig();

// Save configuration
await saveConfig({
  models_directory: "C:/models",
  theme: "dark",
  language: "en",
  max_tokens: 2048,
  temperature: 0.7,
  auto_save_chat: true,
  chat_history_limit: 50
});

// Reset to defaults
const defaultConfig = await resetConfig();

// Get config file path
const path = await getConfigPath();
```

### Backend (Rust)

Commands available:
- `load_config` - Load configuration from disk
- `save_config` - Save configuration to disk
- `reset_config` - Reset to default values
- `get_config_path_string` - Get the config file path

## Default Values

```javascript
{
  models_directory: null,
  theme: 'dark',
  language: 'en',
  max_tokens: 2048,
  temperature: 0.7,
  auto_save_chat: true,
  chat_history_limit: 50
}
```

## Integration with Other Features

### Models Integration

The `models_directory` setting is used by the model scanning system to locate and parse Ollama models.

### Theme System

The theme setting can be used to dynamically switch between color schemes (implementation pending).

### Language Support

The language setting prepares the app for internationalization (i18n implementation pending).

## Files Created

**Backend:**
- `src-tauri/src/commands/config.rs` - Configuration management commands

**Frontend:**
- `src/lib/config.js` - Configuration API wrapper
- `src/routes/settings/+page.svelte` - Settings page UI

**Modified:**
- `src-tauri/src/commands/mod.rs` - Added config module
- `src-tauri/src/lib.rs` - Registered config commands
- `src/routes/+page.svelte` - Added navigation links

## Future Enhancements

- [ ] Theme switching implementation
- [ ] Internationalization (i18n)
- [ ] Export/Import configuration
- [ ] Configuration validation
- [ ] Advanced model parameters
- [ ] Keyboard shortcuts configuration
- [ ] Privacy settings
