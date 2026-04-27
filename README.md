# ISKIN - Intelligent Self-Improving Knowledge Interface Network

Autonomous AI-powered IDE with self-modular architecture, inspired by Devin and Cursor.

## 🚀 Features

- **Modular Self-Improvement**: Hot-reloadable modules (Rust Dylibs + WASM) allowing the agent to upgrade itself without restart
- **Devin-Level Agent**: Full virtual sandbox with VNC, browser automation, and self-healing loops
- **Advanced Memory System**: Hierarchical knowledge base with semantic indexing (Qdrant + Tree-sitter)
- **Global Context Engine**: Real-time code graph for precise impact analysis
- **Secure Tool Execution**: Policy engine with user confirmation for dangerous operations
- **Multimodal Perception**: Vision (Qwen-VL) for UI testing and visual debugging

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
- [ ] Phase 2: Lifecycle Manager & Module System
- [ ] Phase 3: Memory & Context Engine
- [ ] Phase 4: Sandbox & VNC Integration
- [ ] Phase 5: Autonomous Agent & Self-Healing
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
