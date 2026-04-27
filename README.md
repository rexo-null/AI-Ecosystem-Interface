# ISKIN - Intelligent Self-Improving Knowledge Interface Network

Autonomous AI-powered IDE with self-modular architecture, inspired by Devin and Cursor.

## 🚀 Quick Start

### Automated Installation

**Windows:**
```bash
install.bat
```

**Linux/macOS:**
```bash
chmod +x install.sh
./install.sh
```

### Manual Installation

#### Prerequisites
- **Node.js 18+** - [Download](https://nodejs.org/)
- **Rust** - [Install](https://rustup.rs/)
- **System Dependencies:**
  - Linux: `sudo apt install pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev`
  - macOS: `brew install gtk+3 webkit2gtk`
  - Windows: Visual Studio Build Tools with C++ workload

#### Setup Steps
```bash
# Install dependencies
npm install

# Check Rust setup
cd src-tauri
cargo check
cd ..

# Download models (when available)
# Models will be auto-downloaded on first use
```

### Running ISKIN

**Development Mode:**
```bash
# Windows
launch-dev.bat

# Linux/macOS
./launch-dev.sh
```

**Production Build:**
```bash
# Windows
build-release.bat

# Linux/macOS
./build-release.sh
```

## 🏗️ Architecture

```
ISKIN/
├── src-tauri/              # Rust Core (Immutable)
│   ├── src/
│   │   ├── core/           # Lifecycle Manager, Security, Resources
│   │   ├── modules/        # Dynamic Module Loader (Dylib/WASM)
│   │   ├── memory/         # Vector DB, Indexing, Rules Engine
│   │   ├── sandbox/        # Docker, VNC, Browser Automation
│   │   ├── tools/          # MCP-like Tool Registry
│   │   └── api/            # Tauri Commands
│   └── Cargo.toml
├── src/                    # React Frontend
│   ├── components/         # UI Components (Editor, Chat, FileTree, VNC)
│   ├── store/              # State Management
│   └── App.tsx
├── rules/                  # Constitution & Protocols
├── tools/                  # Custom Tool Definitions
└── models/                 # Local LLM Weights
```

## 🛠️ Tech Stack

- **Core**: Rust + Tauri v2
- **AI**: llama.cpp, Qwen-2.5-Coder-14B, Qwen-VL
- **Memory**: Qdrant, Tree-sitter
- **Sandbox**: Docker/Podman, KasmVNC, Playwright
- **Frontend**: React, Monaco Editor, Xterm.js

## 📋 Roadmap

- [x] Phase 1: Project Structure & Core Setup
- [x] Phase 2: Lifecycle Manager & Module System
- [ ] Phase 3: Memory & Context Engine (Qdrant + Tree-sitter)
- [ ] Phase 4: Sandbox & VNC Integration (Docker + KasmVNC)
- [ ] Phase 5: Autonomous Agent & Self-Healing
- [ ] Phase 6: Advanced Security & Policy Engine

## 🔧 Development

### Project Structure
- `src/` - React frontend with Monaco Editor
- `src-tauri/` - Rust backend with Tauri
- `src-tauri/src/core/` - Lifecycle management
- `src-tauri/src/modules/` - Dynamic module system
- `src-tauri/src/api/` - Tauri command handlers

### Key Features Implemented
- **Modular Architecture**: Hot-reloadable Rust modules
- **WASM Support**: Secure module execution via Wasmtime
- **Monaco Editor**: VS Code-quality code editing
- **Terminal Integration**: Xterm.js for shell access
- **File System**: Full file operations via Tauri
- **State Management**: Zustand for React state
- **Localization**: i18n support (RU/EN)

### Adding New Modules
1. Implement `ISKINModule` trait in `src-tauri/src/modules/`
2. Register in `LifecycleManager::new()`
3. Add Tauri commands in `src-tauri/src/api/commands.rs`
4. Update frontend components as needed

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test` and `npm test`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

Inspired by Devin AI, Cursor, and the broader AI IDE community.
- [ ] Phase 6: Security & Polish

## 🚦 Getting Started

```bash
# Install dependencies
cd src-tauri && cargo build
cd ../src && npm install

# Run development
npm run tauri dev
```

## 📜 License

MIT
