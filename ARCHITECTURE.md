# ISKIN — Архитектура и детальный план реализации

## Видение проекта

**ISKIN** (Intelligent Self-Improving Knowledge Interface Network) — автономный AI-агент уровня Devin, работающий как десктопное приложение на ПК пользователя. Ключевая особенность — **модульная self-improving архитектура**: агент может в runtime улучшать собственные модули через hot-reload без перезагрузки ядра.

### Аналогия
Робот отсоединяет руку → улучшает её → подключает обратно. Ядро работает непрерывно, модули обновляются на лету.

---

## Архитектура системы

```
┌─────────────────────────────────────────────────────────────────┐
│                      ISKIN Desktop App (Tauri v2)               │
├─────────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript)                                  │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ FileTree │ │ Monaco    │ │ ChatPanel│ │ TerminalPanel    │  │
│  │ (дерево  │ │ Editor    │ │ (чат с   │ │ (xterm.js + PTY) │  │
│  │ проекта) │ │ (редактор)│ │ агентом) │ │                  │  │
│  ├──────────┤ ├───────────┤ ├──────────┤ ├──────────────────┤  │
│  │Knowledge │ │ CodeSearch│ │ Sandbox  │ │ AgentStatusBar   │  │
│  │ Base UI  │ │           │ │ Panel    │ │ (состояния)      │  │
│  └──────────┘ └───────────┘ └──────────┘ └──────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC Bridge (invoke / events)                             │
├─────────────────────────────────────────────────────────────────┤
│  Rust Core (Immutable Kernel)                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ LifecycleManager — загрузка/выгрузка/hot-reload модулей │    │
│  │ PolicyEngine — security (Strict/Balanced/Permissive)    │    │
│  │ ResourceManager — CPU/RAM мониторинг                    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  Hot-Reload Modules (Dylib .so/.dll/.dylib + WASM .wasm)       │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐      │
│  │ AgentModule   │ │ ToolModule   │ │ MemoryModule       │      │
│  │ (state machine│ │ (tool registry│ │ (knowledge base,  │      │
│  │  планирование │ │  file/shell/  │ │  vector store,    │      │
│  │  кодинг,      │ │  network)     │ │  tree-sitter)     │      │
│  │  тестирование)│ │               │ │                   │      │
│  └──────────────┘ └──────────────┘ └────────────────────┘      │
│                                                                 │
│  System Services                                                │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐      │
│  │ LLM Client   │ │ Sandbox      │ │ Self-Healing       │      │
│  │ (llama.cpp   │ │ (Docker API, │ │ (мониторинг,       │      │
│  │  HTTP API)   │ │  VNC, Browser)│ │  авто-рестарт)     │      │
│  └──────────────┘ └──────────────┘ └────────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│  External Processes                                              │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐      │
│  │ llama-server  │ │ Docker       │ │ Shell (PTY)        │      │
│  │ (Qwen2.5-    │ │ Daemon       │ │                    │      │
│  │  Coder-14B)  │ │              │ │                    │      │
│  └──────────────┘ └──────────────┘ └────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Hot-Reload (Self-Improvement) — как это работает

### Уже реализовано (Фаза 1-2):
- `LifecycleManager` — загрузка `.so`/`.dll`/`.wasm` модулей из `modules/`
- `reload_module()` — выгрузка старого + загрузка нового модуля
- `ISKINModule` trait — единый интерфейс: `initialize()`, `shutdown()`, `execute()`
- `PolicyEngine` — контроль опасных действий (включая `ModuleReload`)

### Что нужно доработать:
1. **Module Compiler** — агент генерирует Rust-код модуля → компилирует в `.so` → `LifecycleManager.reload_module()`
2. **Module Versioning** — хранить версии модулей, rollback при ошибке компиляции
3. **Module API Contract** — стабильный ABI между ядром и модулями (через `#[repr(C)]` или WASM interface types)
4. **Sandbox Compilation** — компиляция модулей в Docker-контейнере (безопасность)

### Цикл self-improvement:
```
1. Агент обнаруживает проблему в модуле (ошибка, низкая производительность)
2. Агент генерирует улучшенный код модуля через LLM
3. Компиляция нового кода в песочнице (cargo build --release)
4. Тестирование нового модуля в песочнице
5. PolicyEngine проверяет безопасность (Strict mode → запрос подтверждения)
6. LifecycleManager.reload_module() — горячая замена
7. Мониторинг нового модуля, rollback при ошибках
```

---

## Анализ рисков и меры защиты

### Риск 1: Hot-Reload Self-Improvement — "двойной меч"

**Угроза:** Агент может войти в бесконечный цикл саморефакторинга, дегенерировать поведение, или обойти PolicyEngine через модифицированный код.

**Архитектурные меры:**

1. **Immutable Core** — LifecycleManager, PolicyEngine, State Machine НИКОГДА не перезагружаются. Hardcoded boundary: модуль не может модифицировать файлы из `src-tauri/src/core/`.

2. **Pre-reload validation pipeline:**
   ```
   Новый модуль → Компиляция в Docker → Автотесты → Format check → 
   → Snapshot текущего состояния → PolicyEngine.requires_confirmation() →
   → reload_module() → Мониторинг 60 сек → OK или auto-rollback
   ```

3. **Snapshot Rollback** — перед каждым reload сохраняется `.so`/`.wasm` предыдущей версии. При crash, panic, или деградации метрик — автоматический откат.

4. **Dry-run режим** — для критических модулей (AgentModule, ToolModule): новая версия запускается параллельно, результаты сравниваются со старой, замена только при подтверждении.

5. **Кворум на критические изменения** — если модуль затрагивает безопасность или ядро → обязательное подтверждение пользователя (даже в Permissive mode).

Референс: LangGraph checkpoint/state rollback модель.

### Риск 2: PolicyEngine — обход через косвенные формулировки

**Угроза:** LLM обходит текстовые фильтры через разбитые команды (`r` + `m` + ` -rf /`), косвенные формулировки, или многошаговые атаки.

**Архитектурные меры:**

1. **Валидация структуры, не текста** — PolicyEngine проверяет JSON Schema вызова инструмента (имя тула, типы аргументов, допустимые значения), а не текст промпта.

2. **Строгие JSON Schema для каждого инструмента:**
   ```rust
   // Каждый tool call проходит через:
   ToolSchema::validate(tool_name, args) → Ok(validated_args) | Err(tool_validation_error)
   // Агент получает tool_validation_error, а не ошибку исполнения
   ```

3. **Allowlist/Denylist на уровне Docker:**
   - seccomp профили (запрет системных вызовов)
   - AppArmor/SELinux для ограничения файловой системы
   - Network policy: только localhost и разрешённые домены

4. **Цепочка валидации:**
   ```
   LLM output → JSON parse → Schema validation → PolicyEngine check → 
   → Resource limits check → Execute in sandbox → Return result
   ```

### Риск 3: Qwen 2.5 Coder 14B + llama.cpp — подводные камни

**Проблема 1: Tool calling — невалидный JSON.**
LLM может генерировать JSON с лишним текстом, незакрытыми скобками, или комментариями.

**Решение:**
- Regex extraction JSON из ответа (`\{.*\}` greedy match)
- Fallback: повторный запрос с инструкцией "respond ONLY with valid JSON"
- Максимум 3 retry, затем ошибка пользователю

**Проблема 2: Context window — деградация при >32K.**

**Решение:**
- Дефолтный context: 8192 токенов в `config/llm.toml`
- Summarization: при превышении лимита — сжатие истории через LLM
- Sliding window: хранить последние N сообщений + system prompt

**Проблема 3: Квантование — баланс качество/скорость.**

**Решение:**
- Q4_K_M (~9.2 GB) — дефолт, оптимальный баланс
- Q5_K_M (~10.5 GB) — опция для пользователей с >16GB VRAM
- Настраивается в `config/llm.toml: quantization`

---

## Состояния агента (State Machine)

```
                    ┌──────────┐
                    │  IDLE    │ ← начальное состояние
                    └────┬─────┘
                         │ пользователь даёт задачу
                    ┌────▼─────┐
                    │ ANALYZING│ ← анализ задачи, чтение контекста
                    └────┬─────┘
                         │
                    ┌────▼─────┐
                    │ PLANNING │ ← декомпозиция на шаги, todo list
                    └────┬─────┘
                         │ подтверждение плана (опционально)
                    ┌────▼─────┐
              ┌────►│ CODING   │ ← генерация/редактирование кода
              │     └────┬─────┘
              │          │
              │     ┌────▼─────┐
              │     │ TESTING  │ ← запуск тестов в песочнице
              │     └────┬─────┘
              │          │
              │          ├── тесты не прошли ──► CODING (цикл)
              │          │
              │     ┌────▼─────┐
              │     │ REVIEWING│ ← self-review кода, lint, typecheck
              │     └────┬─────┘
              │          │
              │          ├── найдены проблемы ──► CODING (цикл)
              │          │
              │     ┌────▼──────┐
              │     │ REPORTING │ ← отчёт пользователю (было/стало)
              │     └────┬──────┘
              │          │
              │     ┌────▼──────────┐
              └─────│SELF_IMPROVING │ ← улучшение собственных модулей
                    └────┬──────────┘
                         │
                    ┌────▼─────┐
                    │  IDLE    │
                    └──────────┘
```

### Инструменты агента (tools):
| Инструмент | Описание | Реализация |
|------------|----------|------------|
| `file_read` | Чтение файлов | `ToolExecutor::handle_file_read` |
| `file_write` | Запись файлов | `ToolExecutor::handle_file_write` |
| `file_list` | Список файлов/каталогов | Новый handler |
| `file_delete` | Удаление файлов (через PolicyEngine) | Новый handler |
| `shell_exec` | Выполнение shell-команд | `ToolExecutor::handle_shell_exec` |
| `search_code` | Поиск по коду (Tree-sitter) | `SemanticIndexer` |
| `search_knowledge` | Поиск в базе знаний | `KnowledgeBase` |
| `docker_exec` | Команда в Docker-контейнере | `ContainerManager` |
| `browser_navigate` | Навигация в headless Chrome | `BrowserAutomation` |
| `module_reload` | Hot-reload модуля | `LifecycleManager` |

---

## Дорожная карта (Phases)

### Фаза 1: Foundation ✅ (Завершена)
- Структура проекта, Tauri v2
- LifecycleManager (hot-reload Dylib/WASM)
- PolicyEngine (3 уровня безопасности)
- ResourceManager
- React frontend (VS Code layout)
- Zustand store, i18n (RU/EN)

### Фаза 2: Memory & Context Engine ✅ (Завершена)
- KnowledgeBase с JSON-персистентностью и полнотекстовым поиском
- SemanticIndexer (Tree-sitter: Rust, TypeScript, Python)
- VectorStore (Qdrant + локальный TF-IDF fallback)
- RulesEngine (regex/glob/contains, приоритеты, JSON-хранилище)
- Knowledge Hub UI, CodeSearch UI
- 18 Tauri-команд

### Фаза 3: Sandbox Environment ✅ (Завершена)
- ContainerManager (Docker API через bollard)
- VncManager (KasmVNC WebSocket proxy)
- BrowserAutomation (Chrome CDP)
- SelfHealingLoop (мониторинг, авто-рестарт, паттерны ошибок)
- SandboxPanel UI
- 40 Tauri-команд

### Фаза 4: LLM Integration (Следующая)
**Цель:** Подключить реальную LLM через чистый llama.cpp

#### 4.1 Backend: LLM Client
- Убрать candle-core/candle-transformers/tokenizers из Cargo.toml
- Переписать `llm.rs` → HTTP-клиент к llama.cpp server API:
  - `POST /completion` — генерация текста
  - Streaming через SSE (Server-Sent Events)
  - System prompt, temperature, max_tokens, stop tokens
  - Conversation history management
- Новые Tauri-команды:
  - `chat_with_llm` (обновить существующую)
  - `llm_status` — статус llama-server
  - `llm_stop_generation` — прервать генерацию

#### 4.2 Frontend: Streaming Chat
- Обновить `ChatPanel.tsx`:
  - Streaming отображение ответов (посимвольно)
  - Markdown рендеринг ответов (уже есть react-markdown)
  - Code blocks с подсветкой синтаксиса
  - Кнопка "Копировать код" / "Вставить в редактор" / "Выполнить в терминале"
  - Индикатор загрузки и кнопка "Стоп"

#### 4.3 Скрипты установки
- `scripts/setup.sh` (Linux/macOS):
  - Установка Rust toolchain (rustup)
  - Установка системных зависимостей (libgtk3, libwebkit2gtk-4.1)
  - Клонирование и сборка llama.cpp из GitHub
  - Скачивание GGUF модели Qwen2.5-Coder-14B (huggingface-cli)
  - npm install
- `scripts/setup.bat` (Windows):
  - Аналог для Windows
- `scripts/run.sh` / `scripts/run.bat`:
  - Запуск llama-server с правильными параметрами
  - Запуск `cargo tauri dev`
- Обновить README.md

#### 4.4 Конфигурация
- `config/llm.toml` — настройки LLM:
  ```toml
  [llm]
  model_path = "models/qwen2.5-coder-14b-instruct-q5_k_m.gguf"
  host = "127.0.0.1"
  port = 8080
  context_length = 8192
  gpu_layers = -1  # все слои на GPU, 0 = CPU only
  threads = 8
  ```

### Фаза 5: Interactive Terminal
**Цель:** Полноценный интерактивный терминал в UI

#### 5.1 Backend: PTY Manager
- Новый модуль `src-tauri/src/terminal/mod.rs`:
  - Spawn PTY процесс (bash/powershell)
  - Чтение stdout/stderr через async stream
  - Запись stdin
  - Resize PTY
  - Множественные сессии (tabs)
- Tauri-команды:
  - `terminal_create` — создать сессию
  - `terminal_write` — отправить input
  - `terminal_resize` — изменить размер
  - `terminal_close` — закрыть сессию
- Tauri Events:
  - `terminal-output` — streaming output через события

#### 5.2 Frontend: xterm.js
- Переписать `TerminalPanel.tsx`:
  - Инициализация xterm.js Terminal
  - Подключение к Tauri PTY через invoke + events
  - Поддержка ANSI escape codes, цвета
  - Вкладки (множественные терминалы)
  - Copy/paste

### Фаза 6: Autonomous Agent (Agent State Machine)
**Цель:** Полноценный автономный агент с состояниями

#### 6.1 Agent Core
- Обновить `agent_module.rs` → State Machine:
  - Состояния: Idle → Analyzing → Planning → Coding → Testing → Reviewing → Reporting → SelfImproving
  - Transition rules: какие переходы разрешены
  - Контекст задачи: текущая цель, план, прогресс
- Agent Loop:
  ```
  while task not complete:
    1. Analyze context (read files, search code)
    2. Plan next step (LLM → structured output)
    3. Execute step (tool call: file_write, shell_exec, etc.)
    4. Validate result (check output, run tests)
    5. Report progress to user
  ```

#### 6.2 Tool Use Protocol
- LLM вызывает инструменты через structured output (JSON):
  ```json
  {
    "thought": "Нужно создать файл calc.py",
    "action": "file_write",
    "args": { "path": "calc.py", "content": "def add(a, b): ..." }
  }
  ```
- Agent парсит JSON → вызывает ToolModule → передаёт результат обратно в LLM
- PolicyEngine проверяет каждое действие

#### 6.3 Frontend: Agent UI
- `AgentStatusBar` — текущее состояние (Planning/Coding/Testing...)
- `TaskPanel` — список задач и подзадач с прогрессом
- Подтверждение опасных действий (модальное окно)
- Streaming log действий агента

### Фаза 7: Self-Improvement (Hot-Reload в действии)
**Цель:** Агент может улучшать собственные модули

#### 7.1 Module Compiler
- Агент генерирует Rust-код нового/улучшенного модуля
- Компиляция в песочнице (Docker):
  ```
  cargo build --release --target-dir /tmp/module_build
  ```
- Копирование `.so` в `modules/`
- `LifecycleManager.reload_module()`

#### 7.2 Module Versioning
- `modules/versions.json` — история версий каждого модуля
- Автоматический rollback при crash после reload
- A/B тестирование: старый vs новый модуль

#### 7.3 WASM Modules (безопасный вариант)
- Для модулей, которым не нужен системный доступ
- Компиляция Rust → WASM (wasm32-wasi target)
- Sandbox execution через Wasmtime (уже подключён)
- Стабильный ABI через WASM interface types

#### 7.4 Self-Improvement Scenarios:
1. **Улучшение промптов** — агент анализирует качество своих ответов, обновляет system prompt
2. **Улучшение инструментов** — агент замечает, что file_search медленный, оптимизирует модуль
3. **Новые инструменты** — агент создаёт новый модуль (например, "git_module.wasm") и загружает его
4. **Исправление багов** — агент обнаруживает ошибку в своём модуле, генерирует fix, компилирует и перезагружает

### Фаза 8: Security Hardening
**Цель:** Безопасная работа агента на ПК пользователя

- PolicyEngine → полноценная RBAC система
- Sandboxed execution: все опасные операции только в Docker
- Audit log: полная история действий агента
- Rate limiting на LLM-запросы
- Ограничение файловой системы (whitelist директорий)
- Подтверждение пользователем для: удаление файлов, sudo, network, module reload

### Фаза 9: Polish & Release
- Performance profiling и оптимизация
- Кросс-платформенная сборка (Linux, macOS, Windows)
- Auto-updater для приложения
- Документация пользователя
- Стабильный API для custom модулей
- Alpha → Beta → Release

---

## Текущее состояние компонентов

| Компонент | Статус | Файл(ы) |
|-----------|--------|---------|
| LifecycleManager (hot-reload) | Реализован (Dylib + WASM) | `core/lifecycle.rs` |
| PolicyEngine (security) | Реализован (3 уровня) | `core/security.rs` |
| ResourceManager | Реализован | `core/resources.rs` |
| KnowledgeBase | Реализован (JSON + full-text) | `memory/knowledge_base.rs` |
| SemanticIndexer (Tree-sitter) | Реализован (Rust/TS/Python) | `memory/indexer.rs` |
| VectorStore (Qdrant fallback) | Реализован | `memory/vector_store.rs` |
| RulesEngine | Реализован | `memory/rules_engine.rs` |
| ContainerManager (Docker) | Реализован (bollard API) | `sandbox/container.rs` |
| VncManager | Реализован | `sandbox/vnc.rs` |
| BrowserAutomation (CDP) | Реализован | `sandbox/browser.rs` |
| SelfHealingLoop | Реализован | `sandbox/self_healing.rs` |
| LLM Engine | **Placeholder** → нужен llama.cpp HTTP client | `llm.rs` |
| TerminalPanel | **Статический HTML** → нужен xterm.js + PTY | `TerminalPanel.tsx` |
| AgentModule | **Базовый** → нужна State Machine | `modules/agent_module.rs` |
| ToolModule | Реализован (file_read/write/run) | `modules/tool_module.rs` |
| Module Compiler | **Не начат** | — |
| Agent UI (status, tasks) | **Не начат** | — |
| Скрипты установки | **Не начат** | — |

---

## Приоритет реализации

```
Фаза 4 (LLM) ──► Фаза 5 (Terminal) ──► Фаза 6 (Agent) ──► Фаза 7 (Self-Improvement)
     │                   │                    │                      │
  llama.cpp           xterm.js            State Machine         Module Compiler
  HTTP client         PTY backend         Tool Use Protocol     WASM modules
  Streaming chat      Multiple tabs       Agent Loop            Versioning
  Setup scripts                           Agent UI              Rollback
```

Каждая фаза строится на предыдущей. Фаза 4 — фундамент для всего остального.

---

## Технический стек (финальный)

| Слой | Технология | Назначение |
|------|-----------|------------|
| Desktop Runtime | Tauri v2 | Нативное приложение с web UI |
| Backend | Rust (tokio async) | Ядро, модули, API |
| Frontend | React 18 + TypeScript | UI компоненты |
| State Management | Zustand | Состояние frontend |
| Code Editor | Monaco Editor | Редактирование файлов |
| Terminal | xterm.js + PTY | Интерактивный терминал |
| LLM | llama.cpp (server mode) | Qwen2.5-Coder-14B (GGUF) |
| Memory | Qdrant + Tree-sitter | Семантический поиск, индексация кода |
| Sandbox | Docker/Podman + bollard | Безопасное выполнение, тестирование |
| VNC | KasmVNC | Визуальный доступ к контейнерам |
| Browser | Headless Chrome (CDP) | Автоматизация браузера |
| Hot-Reload | libloading + Wasmtime | Dylib (.so/.dll) и WASM модули |
| Security | PolicyEngine (RBAC) | Контроль действий агента |
| i18n | Custom (RU/EN) | Локализация |
