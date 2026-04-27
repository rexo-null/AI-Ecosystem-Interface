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

## Phase 3: Sandbox Environment (Weeks 5-7)
- [ ] Docker/Podman integration
- [ ] VNC streaming
- [ ] Browser automation (Playwright)
- [ ] Headless Chrome for frontend testing
- [ ] Self-healing loop

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

## Current Status: Phase 3 Pending

### Completed (Phase 1 + 2):
- Repository structure & Tauri v2 configuration
- Rust core modules (Lifecycle, Security, Resources, PolicyEngine)
- React frontend (VS Code-like layout)
- KnowledgeBase with JSON persistence and full-text search
- SemanticIndexer with Tree-sitter (Rust, TypeScript, Python code parsing)
- VectorStore (Qdrant + local fallback with TF-IDF vectorizer)
- RulesEngine (Constitution/Protocol/UserRule priorities, regex/glob/contains matching, JSON storage)
- 18 Tauri commands (ping, knowledge CRUD, code indexing, rules management, LLM chat, tool/memory/agent modules)
- CodeSearch UI component (project indexing, symbol search, file preview)
- Knowledge Hub UI (search bar with debounce, expandable cards, type filters, content preview)
- Frontend-backend integration via Tauri invoke with localStorage fallback
- i18n (RU/EN) with new sidebar tab and code search translations

### Next Steps:
1. Docker/Podman container management (Sandbox)
2. VNC streaming integration (KasmVNC)
3. Browser automation with Playwright
4. Self-healing loop implementation
