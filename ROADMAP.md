# ISKIN Development Roadmap

> Конечная цель: автономный self-improving AI-агент уровня Devin, работающий локально на ПК пользователя.

## Phase 1: Foundation (Weeks 1-2) — DONE
- [x] Project structure setup (Tauri v2 + React 18 + TypeScript)
- [x] Core modules: LifecycleManager (Dylib/WASM hot-reload), PolicyEngine (3 уровня), ResourceManager
- [x] React frontend (VS Code-like layout: sidebar, editor, chat, terminal)
- [x] Zustand state management, i18n (RU/EN)
- [x] Basic Tauri commands (7)

## Phase 2: Memory & Context Engine (Weeks 3-4) — DONE
- [x] KnowledgeBase (JSON persistence, full-text search, access counting)
- [x] SemanticIndexer (Tree-sitter: Rust, TypeScript, Python code parsing)
- [x] VectorStore (Qdrant + local TF-IDF fallback)
- [x] RulesEngine (regex/glob/contains, priorities, JSON storage)
- [x] Knowledge Hub UI, CodeSearch UI
- [x] 18 Tauri commands

## Phase 3: Sandbox Environment (Weeks 5-7) — DONE
- [x] ContainerManager (Docker API via bollard: create/start/stop/exec/logs, simulation mode)
- [x] VncManager (KasmVNC WebSocket proxy, session management, screenshots)
- [x] BrowserAutomation (Headless Chrome via CDP: navigate/screenshot/JS/action sequences)
- [x] SelfHealingLoop (health monitoring, auto-restart, error pattern detection OOM/CrashLoop/Network/Disk)
- [x] SandboxPanel UI (3 tabs: Containers, Browser, Health)
- [x] 40 Tauri commands total

## Phase 4: LLM Integration (Weeks 8-10) — IN PROGRESS
- [ ] Убрать candle-core (не компилируется, не используется) → HTTP-клиент к llama.cpp
- [ ] `llm.rs`: POST /completion, streaming SSE, system prompt, temperature, stop tokens
- [ ] Streaming ChatPanel: посимвольный вывод, markdown, code blocks с подсветкой
- [ ] Кнопки: "Копировать код" / "Вставить в редактор" / "Выполнить в терминале"
- [ ] `scripts/setup.sh` / `setup.bat`: установка Rust, llama.cpp (из исходников), модель Qwen2.5-Coder-14B GGUF
- [ ] `scripts/run.sh` / `run.bat`: запуск llama-server + cargo tauri dev
- [ ] `config/llm.toml`: model_path, host, port, context_length, gpu_layers, threads
- [ ] `llm_status`, `llm_stop_generation` Tauri commands

## Phase 5: Interactive Terminal (Weeks 11-12)
- [ ] PTY Manager (`src-tauri/src/terminal/mod.rs`): spawn bash/powershell, async I/O, resize
- [ ] xterm.js integration в TerminalPanel.tsx (ANSI escape codes, цвета, copy/paste)
- [ ] Multiple terminal tabs (sessions)
- [ ] Tauri commands: `terminal_create`, `terminal_write`, `terminal_resize`, `terminal_close`
- [ ] Tauri Events: `terminal-output` для streaming

## Phase 6: Autonomous Agent — Devin-level (Weeks 13-16)
Агент с полным функционалом Devin: планирование, кодинг, тестирование, работа с файлами и консолью.

### 6.1 Agent State Machine
- [ ] States: Idle → Analyzing → Planning → Coding → Testing → Reviewing → Reporting → SelfImproving
- [ ] Transition rules, контекст задачи (цель, план, прогресс)
- [ ] Agent Loop: analyze → plan → execute → validate → iterate

### 6.2 Tool Use Protocol
- [ ] LLM → structured JSON output → tool call → result → LLM
- [ ] Tools: file_read, file_write, file_list, file_delete, shell_exec, search_code, search_knowledge, docker_exec, browser_navigate, module_reload
- [ ] PolicyEngine проверка каждого действия

### 6.3 Agent UI
- [ ] AgentStatusBar (текущее состояние: Planning/Coding/Testing...)
- [ ] TaskPanel (список задач с прогрессом)
- [ ] Action log (streaming лог действий)
- [ ] Confirmation dialogs для опасных операций

## Phase 7: Self-Improvement / Hot-Reload (Weeks 17-20)
Ключевая фишка: агент улучшает собственные модули в runtime без перезагрузки ядра.

### 7.1 Module Compiler
- [ ] Агент генерирует Rust-код модуля через LLM
- [ ] Компиляция в Docker-песочнице: `cargo build --release` → `.so`/`.dll`
- [ ] `LifecycleManager.reload_module()` — горячая замена
- [ ] Компиляция WASM-модулей (wasm32-wasi target) для безопасного sandbox execution

### 7.2 Module Versioning & Rollback
- [ ] `modules/versions.json` — история версий каждого модуля
- [ ] Автоматический rollback при crash после reload
- [ ] A/B тестирование: старый vs новый модуль

### 7.3 Self-Improvement Scenarios
- [ ] Улучшение промптов — анализ качества ответов, обновление system prompt
- [ ] Оптимизация инструментов — агент замечает медленную работу, оптимизирует модуль
- [ ] Создание новых инструментов — генерация нового `tool_module.wasm`, загрузка через LifecycleManager
- [ ] Исправление багов — обнаружение ошибки в модуле → fix → compile → reload

## Phase 8: Security Hardening (Weeks 21-22)
- [ ] PolicyEngine → полноценная RBAC система
- [ ] Sandboxed execution: все опасные операции только в Docker
- [ ] Audit log: полная история действий агента с timestamps
- [ ] File system whitelist (ограничение доступа к директориям)
- [ ] Rate limiting на LLM-запросы
- [ ] Подтверждение пользователем: delete, sudo, network, module reload

## Phase 9: Polish & Release (Weeks 23-26)
- [ ] Cross-platform builds (Linux, macOS, Windows) через Tauri bundler
- [ ] Auto-updater для приложения
- [ ] Performance profiling (Rust: flamegraph, Frontend: React DevTools)
- [ ] User documentation (руководство пользователя)
- [ ] Stable API для custom модулей (Module SDK)
- [ ] Qwen-VL для визуального анализа (скриншоты → описание)
- [ ] Alpha → Beta → v1.0 Release

---

## Current Status: Phase 4 In Progress

### Завершено (Phase 1 + 2 + 3):
- Rust core: LifecycleManager (hot-reload Dylib/WASM), PolicyEngine, ResourceManager
- React frontend: VS Code layout, 4 sidebar tabs (Files, Knowledge, Search, Sandbox)
- Memory: KnowledgeBase + SemanticIndexer + VectorStore + RulesEngine
- Sandbox: ContainerManager + VncManager + BrowserAutomation + SelfHealingLoop
- 40 Tauri commands, i18n (RU/EN), Zustand store

### Следующий шаг:
1. LLM Client (llama.cpp HTTP API) → заменить candle placeholder
2. Streaming chat → code blocks → "Execute in terminal"
3. Setup/run scripts для воспроизводимой установки
