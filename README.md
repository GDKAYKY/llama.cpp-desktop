# Llama.cpp Desktop

A Tauri-based desktop application for managing and running Llama.cpp models locally with a premium UI.

## 📚 Documentation

- **[Project Structure](./docs/PROJECT_STRUCTURE.md)**: Detailed overview of the project layout.
- **[Backend Architecture Standards](./docs/BACKEND_ARCHITECTURE.md)**: Mandatory coding standards for the Rust backend, including the **Centralized Models Standard**.
- **[Model Setup Guide](./docs/MODELS_SETUP_GUIDE.md)**: How to configure and run models.

## 🚀 Quick Start

1.  **Prerequisites**: Install [Rust](https://rustup.rs/) and [Node.js](https://nodejs.org/).
2.  **Dependencies**: `npm install`.
3.  **Run**: `npm run dev`.

## 🛠️ Development

- **Frontend**: Svelte 5 + Vite.
- **Backend**: Rust + Tauri v2.
- **Standards**: All shared Rust models MUST be placed in `src-tauri/src/models/`.

---
*Built with ❤️ by the Llama Desktop Team.*
