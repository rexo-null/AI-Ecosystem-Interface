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

## Phase 4: LLM Integration (Weeks 8-10) — DONE
- [x] Убрать candle-core → HTTP-клиент к llama.cpp (`reqwest` + SSE)
- [x] `llm.rs`: ~600 строк — `LLMEngine`, `LLMConfig`, `ModelProfile`, streaming SSE, conversation history
- [x] `ChatPanel.tsx`: streaming + markdown + code blocks + status indicator
- [x] Кнопка "Выполнить в терминале" для bash/sh/shell code blocks
- [x] `scripts/setup.sh` / `setup.bat`: установка llama.cpp (ROCm/Vulkan), скачивание Qwen2.5-Coder-14B GGUF
- [x] `scripts/run.sh` / `run.bat`: запуск llama-server + cargo tauri dev
- [x] `config/llm.toml`: model_path, host, port, context_length, gpu_layers, model_profile, server_args
- [x] `llm_status`, `llm_stop_generation` Tauri commands
- [x] Атомарная история: user+assistant пушатся только при успешном ответе (без orphaned messages)
- [x] Tauri events: `llm-token`, `llm-done`, `llm-error` с `message_id`

## Phase 5: Interactive Terminal (Weeks 11-12) — DONE
- [x] PTY Manager (`src-tauri/src/terminal/mod.rs`): portable-pty, spawn bash/powershell, `std::sync::Mutex`
- [x] xterm.js integration в TerminalPanel.tsx (ANSI escape codes, цвета, fit addon)
- [x] Multiple terminal tabs (create/close/switch)
- [x] Tauri commands: `terminal_create`, `terminal_write`, `terminal_resize`, `terminal_close`, `terminal_list`
- [x] Tauri Events: `terminal-output` для streaming
- [x] Real PTY resize (MasterPty хранится для TIOCSWINSZ/SIGWINCH)
- [x] Интеграция ChatPanel → Terminal: кнопка "Выполнить" на code blocks

## Phase 6: Autonomous Agent — Devin-level (Weeks 13-18)
Агент с полным функционалом Devin: планирование, анализ последствий, кодинг, тестирование, синхронизация артефактов.

### 6.1 Agent State Machine (расширенный)
- [ ] 9 фаз жизненного цикла задачи:
  ```
  ReceiveTask → Decompose → ImpactAssessment → DryRun →
  Execute → Verify → ArtifactSync → Commit → QueueNext
  ```
- [ ] `AgentPhase` enum со строгими правилами переходов
- [ ] `AgentState`: current_phase, task_queue, conversation_history, project_state
- [ ] Agent Loop с `max_retry=3` и автоматическим fallback

### 6.2 Обязательные правила (Devin-level)
- [ ] `DECOMPOSITION_LIMIT`: задача разбивается на ≤3 шага за один цикл, остальное → подзадачи
- [ ] `IMPACT_BEFORE_ACTION`: перед Execute обязателен ImpactReport (affected_files, doc_sync_needed, tests_to_run, rollback_plan)
- [ ] `DRY_RUN_FIRST`: действия с побочным эффектом (risk > safe) сначала в sandbox
- [ ] `VERIFY_OR_ROLLBACK`: если Verify падает → автоматический откат, агент не идёт дальше
- [ ] `ARTIFACT_SYNC_MANDATORY`: после Execute → DocSyncAgent проверяет и обновляет документацию/планы
- [ ] `CONTEXT_BUDGET`: history > 6K токенов → автосаммаризация (последние 5 шагов + текущий план + критические ошибки)
- [ ] `NO_SILENT_FAILURE`: невалидный tool_call → tool_validation_error в промпт, не молчание

### 6.3 Tool Use Protocol
- [ ] LLM → structured JSON output → schema validation → PolicyEngine check → tool call → result → LLM
- [ ] JSON Schema валидация каждого tool call ДО исполнения (`serde_json` + schema)
- [ ] `tool_validation_error` при невалидном JSON (вместо ошибки исполнения)
- [ ] Tools: file_read, file_write, file_list, file_delete, shell_exec, search_code, search_knowledge, docker_exec, browser_navigate, module_reload
- [ ] PolicyEngine проверка каждого действия

### 6.4 Context Management
- [ ] `ContextCompressor`: автосаммаризация при приближении к лимиту контекста
- [ ] Sliding window: system prompt + последние N сообщений + сжатое резюме ранних
- [ ] Критические факты (текущий план, ошибки) сохраняются при сжатии
- [ ] Длинные логи/выводы → в KnowledgeBase (векторное хранилище), не в историю

### 6.5 Agent UI
- [ ] AgentStatusBar (текущее состояние: ReceiveTask/Decompose/ImpactAssessment/Execute...)
- [ ] TaskPanel (список задач с прогрессом, подзадачи)
- [ ] Action log (streaming лог действий агента с rationale)
- [ ] ImpactReport view (что будет затронуто перед выполнением)
- [ ] Confirmation dialogs для опасных операций

## Phase 7: Self-Improvement / Hot-Reload (Weeks 19-22)
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

## Phase 8: Security Hardening (Weeks 23-24)
- [ ] PolicyEngine → полноценная RBAC система
- [ ] Sandboxed execution: все опасные операции только в Docker
- [ ] Audit log: полная история действий агента с timestamps, хэшами, rationale
- [ ] File system whitelist (ограничение доступа к директориям)
- [ ] Rate limiting на LLM-запросы
- [ ] Подтверждение пользователем: delete, sudo, network, module reload

## Phase 9: Polish & Release (Weeks 25-28)
- [ ] Cross-platform builds (Linux, macOS, Windows) через Tauri bundler
- [ ] Auto-updater для приложения
- [ ] Performance profiling (Rust: flamegraph, Frontend: React DevTools)
- [ ] User documentation (руководство пользователя)
- [ ] Stable API для custom модулей (Module SDK)
- [ ] Qwen-VL для визуального анализа (скриншоты → описание)
- [ ] Alpha → Beta → v1.0 Release

## Phase 10: ISKIN Butler — OS-Native AI Assistant (Future)
Расширение ISKIN из IDE-агента в системного "цифрового дворецкого" с полным доступом к ОС.

### Предпосылки
- Требует завершения Phase 6-8 (рабочий агент + безопасность)
- Выделение общего ядра (`iskin-core`) в отдельный крейт
- Расширение PolicyEngine для системных операций

### Планируемые возможности
- **Системный анализ:** сканирование диска, поиск дубликатов, анализ автозагрузки, мониторинг здоровья системы
- **Контролируемое выполнение:** очистка кэша, временных файлов, управление автозагрузкой (с подтверждением + dry_run + rollback)
- **Проактивные уведомления:** "диск заполнен на 90%", "найдено 5GB дубликатов", "50 программ в автозагрузке"
- **Трёхуровневая безопасность:** `safe` (только чтение) → `confirm` (подтверждение) → `dangerous` (подтверждение + снапшот + rollback)
- **Плагинная архитектура:** тулы как модули с `ToolManifest`, включение/выключение через конфиг
- **UI:** системный трей, чат-окно вне IDE, дашборд с метриками, контекстное меню проводника

### Архитектура (задел)
```
iskin/
├── crates/
│   ├── iskin-core/          # Общее ядро (LLM, Agent, Policy, Tools, State)
│   ├── iskin-ide/           # Продукт 1: IDE (текущий, Phase 1-9)
│   └── iskin-butler/        # Продукт 2: Butler (Phase 10)
└── configs/                 # Общие конфиги: llm.toml, policy.toml, model_profiles/
```

---

## Current Status: Phase 6 Next

### Завершено (Phase 1-5):
- Rust core: LifecycleManager (hot-reload Dylib/WASM), PolicyEngine, ResourceManager
- React frontend: VS Code layout, 4 sidebar tabs (Files, Knowledge, Search, Sandbox)
- Memory: KnowledgeBase + SemanticIndexer + VectorStore + RulesEngine
- Sandbox: ContainerManager + VncManager + BrowserAutomation + SelfHealingLoop
- LLM: HTTP-клиент к llama.cpp с SSE streaming, conversation history, model profiles
- Terminal: xterm.js + portable-pty, мульти-таб, resize, интеграция с ChatPanel
- 40+ Tauri commands, i18n (RU/EN), Zustand store

### Следующий шаг:
1. Agent State Machine (9 фаз: ReceiveTask → ... → QueueNext)
2. Tool Use Protocol (JSON Schema validation → PolicyEngine → execute)
3. Context Management (автосаммаризация, sliding window)
4. Agent UI (status bar, task panel, action log)
