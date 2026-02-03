# Release Process

This document outlines the steps to create a production release for Llama Desktop.

## Prerequisites

- Node.js and npm installed.
- Rust and Cargo installed.
- Windows (currently targeted for NSIS installers).

## Building a Release

To build a production-ready NSIS installer, run:

```bash
npm run tauri build
```

The build process will:
1. Run `npm run build` to compile the SvelteKit frontend.
2. Compile the Rust backend in release mode.
3. Bundle the application into an NSIS installer.

## Output Location

The generated installer can be found at:
`src-tauri/target/release/bundle/nsis/llama-desktop_{version}_x64-setup.exe`

## Configuration

The release configuration is managed in `src-tauri/tauri.conf.json`.

- **Targets**: Currently set to `["nsis"]` to ensure a consistent Windows installer format.
- **Product Name**: `llama-desktop`
- **Identifier**: `com.kayky.llama-desktop`
