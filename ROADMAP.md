# ISKIN Development Roadmap

## Phase 1: Foundation (Weeks 1-2) ✅
- [x] Project structure setup
- [x] Tauri v2 configuration
- [x] Core modules (Lifecycle, Security, Resources)
- [x] Basic React frontend layout
- [x] Tauri build system integration
- [x] Basic file system operations

## Phase 2: Memory & Context (Weeks 3-4) ✅
- [x] Knowledge Base implementation (полная реализация с персистентностью)
- [x] Semantic indexing with Qdrant (VectorStore с локальным fallback)
- [x] Tree-sitter code graph (Rust, TypeScript, Python)
- [x] Rules engine (паттерн-матчинг, приоритеты, JSON-хранилище)
- [x] UI for Knowledge Hub (поиск, фильтры, предпросмотр)

## Phase 3: Sandbox Environment (Weeks 5-7) ✅
- [x] Docker/Podman integration (ContainerManager через bollard API)
- [x] VNC streaming (VncManager с KasmVNC WebSocket proxy)
- [x] Browser automation (BrowserAutomation через Chrome CDP)
- [x] Headless Chrome for frontend testing
- [x] Self-healing loop (мониторинг, авто-рестарт, паттерны ошибок)
- [x] Sandbox UI panel (контейнеры, браузер, здоровье)

## Phase 4: AI Integration (Weeks 8-10)
- [ ] llama.cpp integration
- [ ] Qwen-2.5-Coder-14B setup
- [ ] Qwen-VL for visual analysis
- [ ] Streaming chat interface
- [ ] Prompt management

## Phase 5: Autonomous Agent (Weeks 11-14)
- [ ] Task planning system
- [ ] Tool execution framework
- [ ] Code diff and merge
- [ ] Self-improvement module
- [ ] Hot-reload mechanism

## Phase 6: Polish & Release (Weeks 15-16)
- [ ] Security audit
- [ ] Performance optimization
- [ ] Documentation
- [ ] Alpha release

## Current Status: Phase 4 Pending

### Completed (Phase 1 + 2 + 3):
- Repository structure & Tauri v2 configuration
- Rust core modules (Lifecycle, Security, Resources, PolicyEngine)
- React frontend (VS Code-like layout, 4 sidebar tabs)
- KnowledgeBase with JSON persistence and full-text search
- SemanticIndexer with Tree-sitter (Rust, TypeScript, Python code parsing)
- VectorStore (Qdrant + local fallback with TF-IDF vectorizer)
- RulesEngine (Constitution/Protocol/UserRule priorities, regex/glob/contains matching, JSON storage)
- ContainerManager with Docker API (bollard): create/start/stop/remove, exec, logs, health checks, simulation mode
- VncManager: KasmVNC WebSocket proxy, session management, resolution/quality control, screenshot capture
- BrowserAutomation: Headless Chrome via CDP, navigate/screenshot/execute JS, action sequences
- SelfHealingLoop: container health monitoring, auto-restart, error pattern detection, recovery strategies
- 38 Tauri commands (18 Phase 2 + 20 Phase 3: containers, VNC, browser, self-healing, sandbox status)
- SandboxPanel UI (container list with controls, browser automation, health stats dashboard)
- CodeSearch UI component (project indexing, symbol search, file preview)
- Knowledge Hub UI (search bar with debounce, expandable cards, type filters, content preview)
- Frontend-backend integration via Tauri invoke with localStorage fallback
- i18n (RU/EN)

### Next Steps:
1. LLM integration (llama.cpp + Qwen-2.5-Coder-14B)
2. Streaming chat interface
3. Prompt management system
4. Visual analysis (Qwen-VL)
