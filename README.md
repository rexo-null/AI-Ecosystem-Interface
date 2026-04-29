# ISKIN - Intelligent Self-Improving Knowledge Interface Network

Автономный AI-агент уровня Devin, работающий как десктопное приложение на вашем ПК. Полноценная IDE с AI-ассистентом, который может планировать, писать код, тестировать в песочнице и **улучшать собственные модули** в runtime через hot-reload — без перезагрузки системы.

> Представьте робота, который отсоединяет руку, улучшает её и подключает обратно — ядро продолжает работать непрерывно.

## Ключевые возможности

- **Автономный агент** — состояния: планирование → кодинг → тестирование → отчёт
- **LLM на вашем ПК** — Qwen2.5-Coder-14B через чистый llama.cpp (без облака, без обёрток)
- **Hot-Reload модули** — агент улучшает свои Dylib/WASM-модули на лету
- **Песочница** — Docker-контейнеры для безопасного тестирования
- **IDE интерфейс** — Monaco Editor, File Tree, интерактивный терминал, чат с агентом
- **Память и контекст** — Knowledge Base, Tree-sitter индексация, векторный поиск

## Quick Start

### Автоматическая установка

**Linux/macOS:**
```bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

**Windows:**
```bash
scripts\setup.bat
```

Скрипт установит: Rust, системные зависимости (Tauri v2), llama.cpp, модель Qwen2.5-Coder-14B (GGUF), npm-зависимости.

### Запуск

```bash
# Запуск llama-server + ISKIN
./scripts/run.sh

# Или вручную:
# 1. Запустить LLM сервер
./llama.cpp/build/bin/llama-server \
  -m models/qwen2.5-coder-14b-instruct-q5_k_m.gguf \
  --host 127.0.0.1 --port 8080 \
  -c 8192 -ngl 99 -t 8

# 2. Запустить ISKIN
npm run tauri dev
```

### Ручная установка

#### Предварительные требования
- **Node.js 18+** — [Download](https://nodejs.org/)
- **Rust** — [Install](https://rustup.rs/)
- **llama.cpp** — [GitHub](https://github.com/ggerganov/llama.cpp) (собрать из исходников)
- **Модель** — [Qwen2.5-Coder-14B-Instruct-GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF) (Q5_K_M рекомендуется)
- **Docker** — для песочницы (опционально)
- **Системные зависимости:**
  - Linux: `sudo apt install pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools with C++ workload

```bash
# 1. Установить зависимости
npm install

# 2. Проверить Rust
cd src-tauri && cargo check && cd ..

# 3. Собрать llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp && cmake -B build && cmake --build build --config Release -j$(nproc)
cd ..

# 4. Скачать модель
huggingface-cli download Qwen/Qwen2.5-Coder-14B-Instruct-GGUF \
  qwen2.5-coder-14b-instruct-q5_k_m.gguf \
  --local-dir models/
```

## Архитектура

```
ISKIN/
├── src-tauri/              # Rust Core (Immutable Kernel)
│   ├── src/
│   │   ├── core/           # LifecycleManager (hot-reload), PolicyEngine, Resources
│   │   ├── modules/        # Hot-Reload модули (AgentModule, ToolModule, MemoryModule)
│   │   ├── memory/         # KnowledgeBase, VectorStore (Qdrant), Tree-sitter, RulesEngine
│   │   ├── sandbox/        # Docker (bollard), VNC (KasmVNC), Browser (CDP), Self-Healing
│   │   ├── tools/          # Tool Registry, Tool Executor
│   │   ├── llm.rs          # LLM Client (llama.cpp HTTP API)
│   │   └── api/            # Tauri Commands (40)
│   └── Cargo.toml
├── src/                    # React Frontend
│   ├── components/
│   │   ├── EditorArea.tsx   # Monaco Editor
│   │   ├── ChatPanel.tsx    # Чат с AI агентом
│   │   ├── FileTree.tsx     # Дерево проекта
│   │   ├── TerminalPanel.tsx# Интерактивный терминал (xterm.js)
│   │   ├── KnowledgeBase.tsx# База знаний
│   │   ├── CodeSearch.tsx   # Поиск по коду
│   │   └── SandboxPanel.tsx # Управление песочницей
│   ├── store.ts            # Zustand state management
│   └── App.tsx
├── scripts/                # Скрипты установки и запуска
│   ├── setup.sh            # Linux/macOS установка
│   ├── setup.bat           # Windows установка
│   ├── run.sh              # Linux/macOS запуск
│   └── run.bat             # Windows запуск
├── config/                 # Конфигурация
│   └── llm.toml            # Настройки LLM (модель, порт, GPU layers)
├── models/                 # Локальные LLM модели (GGUF)
├── modules/                # Hot-reload модули (.so/.dll/.wasm)
├── ARCHITECTURE.md         # Детальная архитектура и план
└── ROADMAP.md              # Дорожная карта
```

## Технический стек

| Слой | Технология | Назначение |
|------|-----------|------------|
| Desktop Runtime | Tauri v2 | Нативное приложение |
| Backend | Rust (tokio async) | Ядро, модули, API |
| Frontend | React 18 + TypeScript | UI |
| State | Zustand | Управление состоянием |
| Code Editor | Monaco Editor | Редактор кода |
| Terminal | xterm.js + PTY | Интерактивный терминал |
| LLM | llama.cpp (чистый, без обёрток) | Qwen2.5-Coder-14B |
| Memory | Qdrant + Tree-sitter | Семантический поиск, индексация |
| Sandbox | Docker/Podman + bollard | Песочница |
| VNC | KasmVNC | Визуальный доступ к контейнерам |
| Browser | Headless Chrome (CDP) | Автоматизация |
| Hot-Reload | libloading + Wasmtime | Dylib и WASM модули |
| Security | PolicyEngine (RBAC) | Контроль действий агента |
| i18n | Custom | RU/EN |

## Roadmap

### Phase 1: Foundation — DONE
- [x] Tauri v2 + React frontend (VS Code layout)
- [x] LifecycleManager (Dylib/WASM hot-reload)
- [x] PolicyEngine (Strict/Balanced/Permissive)
- [x] ResourceManager, Zustand store, i18n (RU/EN)

### Phase 2: Memory & Context Engine — DONE
- [x] KnowledgeBase (JSON persistence, full-text search, access counting)
- [x] SemanticIndexer (Tree-sitter: Rust, TypeScript, Python)
- [x] VectorStore (Qdrant + local TF-IDF fallback)
- [x] RulesEngine (regex/glob/contains, priorities, JSON storage)
- [x] Knowledge Hub UI, CodeSearch UI
- [x] 18 Tauri commands

### Phase 3: Sandbox Environment — DONE
- [x] ContainerManager (Docker API via bollard: create/start/stop/exec/logs)
- [x] VncManager (KasmVNC WebSocket proxy, screenshots)
- [x] BrowserAutomation (Chrome CDP: navigate/screenshot/JS/actions)
- [x] SelfHealingLoop (health monitoring, auto-restart, error patterns)
- [x] SandboxPanel UI (3 tabs: Containers, Browser, Health)
- [x] 40 Tauri commands total

### Phase 4: LLM Integration — IN PROGRESS
- [ ] llama.cpp HTTP client (replace candle placeholder)
- [ ] Streaming chat (SSE, markdown rendering, code blocks)
- [ ] Setup scripts (setup.sh/bat, run.sh/bat)
- [ ] LLM config (config/llm.toml)
- [ ] "Copy code" / "Insert to editor" / "Run in terminal" buttons

### Phase 5: Interactive Terminal
- [ ] PTY Manager (spawn bash/powershell, async I/O)
- [ ] xterm.js integration (ANSI, colors, resize)
- [ ] Multiple terminal tabs
- [ ] Tauri commands + events for terminal streaming

### Phase 6: Autonomous Agent (Devin-level)
- [ ] Agent State Machine (Idle → Analyzing → Planning → Coding → Testing → Reviewing → Reporting)
- [ ] Tool Use Protocol (LLM → structured JSON → tool call → result → LLM)
- [ ] Agent Loop (plan → execute → validate → iterate)
- [ ] Agent UI (status bar, task panel, action log, confirmation dialogs)
- [ ] Integration: Agent → LLM → Tools → FileSystem → Terminal → Sandbox

### Phase 7: Self-Improvement (Hot-Reload)
- [ ] Module Compiler (agent generates Rust code → compiles .so in Docker → hot-reload)
- [ ] Module Versioning (version history, automatic rollback on crash)
- [ ] WASM modules (safe sandboxed execution via Wasmtime)
- [ ] Self-improvement scenarios: prompt tuning, tool optimization, bug fixing, new tools

### Phase 8: Security Hardening
- [ ] Full RBAC in PolicyEngine
- [ ] Sandboxed execution for all dangerous operations
- [ ] Audit log (complete action history)
- [ ] File system whitelist, rate limiting
- [ ] User confirmation for: delete, sudo, network, module reload

### Phase 9: Polish & Release
- [ ] Cross-platform builds (Linux, macOS, Windows)
- [ ] Auto-updater
- [ ] Performance profiling and optimization
- [ ] User documentation
- [ ] Stable API for custom modules
- [ ] Alpha → Beta → Release

## Self-Improvement: как это работает

```
1. Агент обнаруживает проблему в модуле (ошибка, медленная работа)
2. Генерирует улучшенный код через LLM (Qwen2.5-Coder-14B)
3. Компилирует новый модуль в Docker-песочнице (cargo build)
4. Тестирует новый модуль в изолированной среде
5. PolicyEngine проверяет безопасность
6. LifecycleManager.reload_module() — горячая замена без перезагрузки
7. Мониторинг, автоматический rollback при ошибках
```

## Agent States

```
IDLE → ANALYZING → PLANNING → CODING → TESTING → REVIEWING → REPORTING
  ↑                              ↑         |          |
  |                              └─────────┘          |
  |                           (тесты не прошли)       |
  └───────────────────────────────────────────────────┘
                        (задача завершена)
```

## Разработка

### Создание нового модуля
1. Реализовать трейт `ISKINModule` в `src-tauri/src/modules/`
2. Скомпилировать как `.so` / `.dll` / `.wasm`
3. Положить в `modules/`
4. Модуль загрузится автоматически через `LifecycleManager`

### Tauri Commands
Все команды определены в `src-tauri/src/api/commands.rs`. Текущий список: 40 команд для Core, Knowledge, Code Search, Rules, LLM, Sandbox, Containers, VNC, Browser, Self-Healing.

## License

MIT
