# Llama Desktop
<img width="10240" height="1024" alt="image" src="https://github.com/user-attachments/assets/980cb5f8-3460-4035-b4c8-b517f5e14249" />

A lightweight **Tauri and Rust** desktop app for running and managing local Large Language Models via llama.cpp.

## Features

<table>
<tr>
<td width="50%">
<img width="100%" height="100%" alt="image" src="https://github.com/user-attachments/assets/367878e8-d767-421f-bfc2-5e30463bb285" />

</td>
<td width="50%">

- Run LLMs locally using llama.cpp
- Model management (download / load / unload)
- Chat interface
- MCPs Management
- Tauri backend for native performance

</td>
</tr>
</table>

## 📦 Install process (Windows)
1. Go to the [Releases page](https://github.com/GDKAYKY/llama.cpp-desktop/releases/latest)
2. Download the installer for your operating system:
   - **Windows**: [Llama Desktop v.1.2.1](https://github.com/GDKAYKY/llama.cpp-desktop/releases/download/v.1.2.1/Llama.Desktop_1.2.1_x64-setup.exe)

1. Run the `.exe`
2. Follow the setup wizard
3. Launch **Llama Desktop**

### Alternatively you can run from Source
```bash
# Clone the repository
git clone https://github.com/GDKAYKY/llama.cpp-desktop.git

# Install dependencies
npm install

# Run the application
npm run tauri dev
```

## 🛠️ Development

### Core

- **Tauri** — Desktop framework (Rust + Web)
- **Rust** — Backend / native bindings
- **Svelte and Tailwind CSS** — Frontend UI
- **Vite.js** — Dev tooling / bundler
- **IndexedDB** - Chat History
- **TypeScript** — Static type checking / type safety

### CI
- SonarCloud
- Cargo Test

### LLM Runtime

- **llama.cpp** — Local LLM inference

### Special Thanks to https://github.com/ggml-org/llama.cpp for making this project possible
