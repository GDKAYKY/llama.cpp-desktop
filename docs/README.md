# Llama Desktop

A Tauri-based desktop application for running Ollama models locally.

[![CI/CD](https://github.com/GDKAYKY/llama.cpp-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/GDKAYKY/llama.cpp-desktop/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/GDKAYKY/llama.cpp-desktop/branch/main/graph/badge.svg)](https://codecov.io/gh/GDKAYKY/llama.cpp-desktop)

## Features

- 🚀 Desktop app built with Tauri v2
- 🦀 Rust backend for model management
- 📦 Ollama model integration
- 🧪 Comprehensive test coverage
- 🔄 CI/CD with GitHub Actions

## Prerequisites

- Node.js 18+ 
- Rust (install from https://rustup.rs/)
- Ollama models in `E:\models` (Windows)

## Installation

```bash
npm install
```

## Development

```bash
npm run dev
```

This will start the Tauri development server with hot reload.

## Testing

Run all tests:
```bash
npm test
```

Watch mode:
```bash
npm run test:watch
```

Coverage report:
```bash
npm run test:coverage
```

## Linting

Check code style:
```bash
npm run lint
```

Fix issues:
```bash
npm run lint:fix
```

## Building

Build for production:
```bash
npm run build
```

## Project Structure

```
├── src/                    # Frontend (HTML/JS/CSS)
├── src-tauri/             # Tauri backend (Rust)
├── test/                  # Test files
├── .github/workflows/     # CI/CD workflows
└── package.json           # Dependencies and scripts
```

## CI/CD

GitHub Actions automatically:
- Runs tests on push/PR
- Tests on multiple OS (Ubuntu, Windows, macOS)
- Tests on multiple Node versions (18.x, 20.x)
- Generates coverage reports
- Builds Tauri app for all platforms
- Uploads artifacts

## License

ISC
