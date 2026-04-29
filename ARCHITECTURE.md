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
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ FileTree │ │ Monaco    │ │ ChatPanel│ │ TerminalPanel    │   │
│  │ (дерево  │ │ Editor    │ │ (чат с   │ │ (xterm.js + PTY) │   │
│  │ проекта) │ │ (редактор)│ │ агентом) │ │                  │   │
│  ├──────────┤ ├───────────┤ ├──────────┤ ├──────────────────┤   │
│  │Knowledge │ │ CodeSearch│ │ Sandbox  │ │ AgentStatusBar   │   │
│  │ Base UI  │ │           │ │ Panel    │ │ (состояния)      │   │
│  └──────────┘ └───────────┘ └──────────┘ └──────────────────┘   │
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
│  Hot-Reload Modules (Dylib .so/.dll/.dylib + WASM .wasm)        │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐       │
│  │ AgentModule  │ │ ToolModule   │ │ MemoryModule       │       │
│  │(state machine│ │(tool registry│ │ (knowledge base,   │       │
│  │ планирование │ │  file/shell/ │ │  vector store,     │       │
│  │ кодинг,      │ │  network)    │ │  tree-sitter)      │       │
│  │ тестирование)│ │              │ │                    │       │
│  └──────────────┘ └──────────────┘ └────────────────────┘       │
│                                                                 │
│  System Services                                                │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐       │
│  │ LLM Client   │ │ Sandbox      │ │ Self-Healing       │       │
│  │ (llama.cpp   │ │ (Docker API, │ │ (мониторинг,       │       │
│  │  HTTP API)   │ │ VNC, Browser)│ │  авто-рестарт)     │       │
│  └──────────────┘ └──────────────┘ └────────────────────┘       │
├─────────────────────────────────────────────────────────────────┤
│  External Processes                                             │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐       │
│  │ llama-server │ │ Docker       │ │ Shell (PTY)        │       │
│  │ (Qwen2.5-    │ │ Daemon       │ │                    │       │
│  │  Coder-14B)  │ │              │ │                    │       │
│  └──────────────┘ └──────────────┘ └────────────────────┘       │
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

## Эволюция модели и мульти-модельная архитектура

### Reference Target (целевой ПК)
- GPU: AMD Radeon RX 6700 XT (12 GB VRAM)
- CPU: AMD Ryzen 5
- RAM: 32 GB DDR4 3600 MHz
- Ограничение: одна модель в VRAM одновременно (~9-10 GB max для Q4_K_M)

### Уровень 1: Одна модель (текущий — Phase 4)

```toml
[llm]
model_path = "models/qwen2.5-coder-14b-instruct-q4_k_m.gguf"
host = "127.0.0.1"
port = 8080
context_length = 8192
gpu_layers = -1
threads = 8
```

Замена модели — "открутил → поставил":
1. Скачать новый `.gguf` (Qwen3, DeepSeek-Coder-V3, Llama 4, и т.д.)
2. Изменить `model_path` в `config/llm.toml`
3. Перезапустить llama-server
4. ISKIN работает с новой моделью без изменений кода

**Почему код не меняется:** `llm.rs` общается через стандартный HTTP API llama.cpp (`/completion`), который одинаков для всех GGUF-моделей.

**Нюанс: Model Profiles.** Разные модели требуют разные system prompts и форматы tool calling. Решение — секция `[model_profile]` в конфиге:

```toml
[model_profile]
name = "qwen2.5-coder"
chat_template = "chatml"        # chatml / llama3 / mistral / custom
tool_call_format = "json_block" # json_block / function_call / xml
system_prompt_file = "prompts/qwen-coder.txt"
stop_tokens = ["<|im_end|>", "<|endoftext|>"]
```

При замене модели пользователь выбирает профиль или создаёт свой. ISKIN подставляет правильные промпты и парсеры автоматически.

### Уровень 2: Hot-Swap моделей (одна GPU)

Для ПК с одной GPU — последовательное переключение между моделями:

```
AgentModule (state: Planning) → нужен planner
  1. ISKIN сохраняет: state, conversation_history, knowledge_context
  2. Останавливает llama-server с coder-14B
  3. Запускает llama-server с planner-32B (или другой моделью)
  4. Восстанавливает контекст через system prompt + history
  5. Планирование завершено

AgentModule (state: Coding) → нужен coder
  6. Останавливает planner
  7. Запускает coder
  8. Восстанавливает контекст
  9. Генерация кода
```

**Контекст НЕ теряется** — он хранится в ISKIN (не в модели):
- `AgentModule` → state, план, задачи, прогресс
- `llm.rs` → conversation_history (все сообщения)
- `KnowledgeBase` → знания и результаты работы

При hot-swap ISKIN передаёт сохранённый контекст в новую модель через system prompt + последние N сообщений из history.

**Trade-off:** Переключение занимает 10-30 секунд (загрузка модели в VRAM). Не подходит для быстрого ping-pong между моделями. Оптимально для длинных фаз (планирование → кодинг).

### Уровень 3: Ансамбль моделей (несколько GPU — будущее)

При апгрейде ПК (2+ GPU) — параллельный запуск нескольких моделей:

```toml
[models.planner]
model_path = "models/qwen3-32b-q4.gguf"
port = 8080
gpu_devices = [0]              # GPU 0
role = "planning, analysis, task decomposition"

[models.coder]
model_path = "models/qwen2.5-coder-14b-q5.gguf"
port = 8081
gpu_devices = [1]              # GPU 1
role = "code generation, editing, refactoring"

[models.reviewer]
model_path = "models/qwen2.5-coder-7b-q4.gguf"
port = 8082
gpu_devices = [1]              # GPU 1 (7B маленькая, влезет рядом с 14B)
role = "code review, bug detection, validation"
```

`AgentModule` маршрутизирует запросы по состоянию:
- **Analyzing/Planning** → planner (большая, умная)
- **Coding** → coder (специализированная на коде)
- **Reviewing** → reviewer (быстрая, для проверки)
- **SelfImproving** → planner + coder (оба)

**Реализация в `llm.rs`:**
```rust
pub struct LLMRouter {
    endpoints: HashMap<String, LLMEndpoint>,  // role → endpoint
    active_model: String,
}

impl LLMRouter {
    // Для Уровня 1: один endpoint
    // Для Уровня 2: hot-swap между endpoints
    // Для Уровня 3: параллельные endpoints
    pub async fn route(&self, role: &str, request: ChatRequest) -> LLMResponse;
}
```

Код `AgentModule` не меняется при переходе между уровнями — меняется только конфиг и стратегия `LLMRouter`.

---

## Состояния агента (State Machine) — Devin-level

### Жизненный цикл задачи (9 фаз)

```
┌─────────────┐
│ ReceiveTask  │ ← пользователь даёт задачу
└──────┬──────┘
       │
┌──────▼──────┐
│  Decompose   │ ← разбивка на ≤3 подзадачи (DECOMPOSITION_LIMIT)
└──────┬──────┘
       │
┌──────▼──────────────┐
│  ImpactAssessment    │ ← ОБЯЗАТЕЛЬНО: affected_files, doc_sync_needed,
└──────┬──────────────┘   tests_to_run, rollback_plan (JSON Schema)
       │
       │ (risk > safe?)
       │
┌──────▼──────┐
│   DryRun     │ ← sandbox execution (опционально для safe actions)
└──────┬──────┘
       │
┌──────▼──────┐
│   Execute    │ ← выполнение действия (tool call)
└──────┬──────┘
       │
┌──────▼──────┐        ┌────────────┐
│   Verify     │──fail─►│  Rollback   │──► Decompose (retry, max_retry=3)
└──────┬──────┘        └────────────┘
       │ pass
┌──────▼──────────┐
│  ArtifactSync    │ ← DocSyncAgent: обновление документации/планов
└──────┬──────────┘
       │
┌──────▼──────┐
│   Commit     │ ← git commit + snapshot
└──────┬──────┘
       │
┌──────▼──────┐
│  QueueNext   │ ← следующая подзадача или завершение → отчёт пользователю
└─────────────┘
```

### Обязательные правила агента (Devin-level)

Эти правила вшиваются в PolicyEngine и State Machine. Они не опциональны.

| # | Правило | Описание | Реализация |
|---|---------|----------|------------|
| 1 | `DECOMPOSITION_LIMIT` | Задача разбивается на ≤3 шага за один цикл. Остальное → подзадачи в очередь | `AgentLoop` разбивает `task` → `subtasks[]`. Если `len > 3` → рекурсия |
| 2 | `IMPACT_BEFORE_ACTION` | Перед Execute обязателен ImpactReport. Без него переход заблокирован | JSON Schema: `affected_files`, `doc_sync_needed`, `tests_to_run`, `rollback_plan` |
| 3 | `DRY_RUN_FIRST` | Действие с побочным эффектом (risk > safe) сначала в sandbox | `ToolExecutor` проверяет `risk_level` через PolicyEngine |
| 4 | `VERIFY_OR_ROLLBACK` | Если Verify (тесты/линтер) падает → автоматический откат | `RollbackManager.restore(snapshot_id)` → возврат в Decompose |
| 5 | `ARTIFACT_SYNC_MANDATORY` | После Execute → DocSyncAgent проверяет и обновляет документацию | Отдельный поток: диффит state → генерирует патчи → коммитит |
| 6 | `CONTEXT_BUDGET` | history > 6K токенов → автосаммаризация | Оставляет: текущий план, последние 5 шагов, критические ошибки |
| 7 | `AUDIT_EVERYTHING` | Каждый LLM-вызов, tool call, решение логируется | `AuditLogger`: время, хэш, rationale, результат |
| 8 | `NO_SILENT_FAILURE` | Невалидный tool_call → tool_validation_error в промпт | `LLMRouter` ловит `parse_error`, возвращает ошибку с примером |

### Context Management (управление контекстом)

14B модель с 8K контекстом требует активного управления историей:

```
Стратегия: Sliding Window + Auto-Summarization

conversation_history > 6K токенов?
  ├── Да → ContextCompressor:
  │        1. Оставить system prompt (фиксированный)
  │        2. Оставить текущий план + активную подзадачу
  │        3. Сжать ранние сообщения в резюме через LLM
  │        4. Оставить последние 5 сообщений полностью
  │        5. Критические ошибки → сохранить всегда
  │        6. Длинные логи/выводы → в KnowledgeBase (векторный поиск)
  └── Нет → использовать как есть
```

Принцип: модель не должна "забывать" цель задачи и текущий прогресс, даже при длинных сессиях.

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

### Фаза 4: LLM Integration ✅ (Завершена)
- `llm.rs` (~600 строк): `LLMEngine`, `LLMConfig`, `ModelProfile`, SSE streaming через `reqwest`
- Атомарная conversation_history (user+assistant пушатся только при success)
- `ChatPanel.tsx`: streaming + markdown + code blocks + status indicator + кнопка "Выполнить"
- `config/llm.toml`: model profiles, server args, оптимизация под RX 6700 XT
- `scripts/setup.sh/bat` + `scripts/run.sh/bat`: автоматическая установка и запуск
- Tauri events: `llm-token`, `llm-done`, `llm-error`
- Tauri commands: `chat_with_llm`, `llm_status`, `llm_stop_generation`

### Фаза 5: Interactive Terminal ✅ (Завершена)
- `terminal/mod.rs`: `TerminalManager` с `portable-pty`, `std::sync::Mutex`
- Кроссплатформенный PTY (bash/powershell), авто-определение `$SHELL`
- Real resize через `MasterPty` (TIOCSWINSZ/SIGWINCH)
- `TerminalPanel.tsx`: xterm.js + fit addon, мульти-таб (create/close/switch)
- Интеграция ChatPanel → Terminal: кнопка "Выполнить" на bash/sh/shell code blocks
- Tauri commands: `terminal_create`, `terminal_write`, `terminal_resize`, `terminal_close`, `terminal_list`
- `is_alive: AtomicBool` — корректное определение завершения процесса

### Фаза 6: Autonomous Agent — Devin-level (Следующая)
**Цель:** Полноценный автономный агент с 9-фазным жизненным циклом

#### 6.1 Agent Core (State Machine)
- `AgentPhase` enum: ReceiveTask → Decompose → ImpactAssessment → DryRun → Execute → Verify → ArtifactSync → Commit → QueueNext
- Строгие правила переходов (см. раздел "Обязательные правила агента")
- `AgentState`: current_phase, task_queue, conversation_history, project_state
- Agent Loop с `max_retry=3` и автоматическим fallback

#### 6.2 Tool Use Protocol
- LLM → structured JSON → Schema validation → PolicyEngine → tool call → result → LLM:
  ```json
  {
    "thought": "Нужно создать файл calc.py",
    "action": "file_write",
    "args": { "path": "calc.py", "content": "def add(a, b): ..." }
  }
  ```
- JSON Schema валидация каждого tool call ДО исполнения
- `tool_validation_error` при невалидном JSON (не ошибка исполнения)
- PolicyEngine проверяет risk_level каждого действия

#### 6.3 Context Management
- `ContextCompressor`: автосаммаризация при history > 6K токенов
- Sliding window: system prompt + сжатое резюме + последние 5 сообщений
- Длинные логи → KnowledgeBase (векторный поиск), не в историю

#### 6.4 Frontend: Agent UI
- `AgentStatusBar` — текущая фаза (ReceiveTask/Decompose/ImpactAssessment/Execute...)
- `TaskPanel` — задачи и подзадачи с прогрессом
- `ImpactReport` view — что будет затронуто перед выполнением
- Action log — streaming лог действий с rationale
- Confirmation dialogs для опасных операций

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
- Audit log: полная история действий агента с timestamps, хэшами, rationale
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

### Фаза 10: ISKIN Butler — OS-Native AI Assistant (Future)
**Цель:** Расширение ISKIN из IDE-агента в системного "цифрового дворецкого"

Требует завершения Phase 6-8. Подробный план в ROADMAP.md.

Ключевые идеи:
- Выделение общего ядра `iskin-core` (LLM, Agent, Policy, Tools, State)
- Два продукта: `iskin-ide` (IDE) + `iskin-butler` (OS-level assistant)
- Системные тулы: сканирование диска, поиск дубликатов, управление автозагрузкой
- Трёхуровневая безопасность: `safe` → `confirm` → `dangerous`
- Плагинная архитектура: `ToolManifest` + `ToolRegistry`
- UI: системный трей, чат-окно, дашборд, контекстное меню проводника

---

## Текущее состояние компонентов

| Компонент | Статус | Файл(ы) |
|-----------|--------|---------|
| LifecycleManager (hot-reload) | ✅ Реализован (Dylib + WASM) | `core/lifecycle.rs` |
| PolicyEngine (security) | ✅ Реализован (3 уровня) | `core/security.rs` |
| ResourceManager | ✅ Реализован | `core/resources.rs` |
| KnowledgeBase | ✅ Реализован (JSON + full-text) | `memory/knowledge_base.rs` |
| SemanticIndexer (Tree-sitter) | ✅ Реализован (Rust/TS/Python) | `memory/indexer.rs` |
| VectorStore (Qdrant fallback) | ✅ Реализован | `memory/vector_store.rs` |
| RulesEngine | ✅ Реализован | `memory/rules_engine.rs` |
| ContainerManager (Docker) | ✅ Реализован (bollard API) | `sandbox/container.rs` |
| VncManager | ✅ Реализован | `sandbox/vnc.rs` |
| BrowserAutomation (CDP) | ✅ Реализован | `sandbox/browser.rs` |
| SelfHealingLoop | ✅ Реализован | `sandbox/self_healing.rs` |
| LLM Engine | ✅ Реализован (llama.cpp HTTP + SSE streaming) | `llm.rs` |
| TerminalManager (PTY) | ✅ Реализован (portable-pty, мульти-таб) | `terminal/mod.rs` |
| TerminalPanel (xterm.js) | ✅ Реализован (fit addon, tabs, resize) | `TerminalPanel.tsx` |
| ChatPanel (streaming) | ✅ Реализован (markdown, code blocks) | `ChatPanel.tsx` |
| Скрипты установки | ✅ Реализованы (ROCm/Vulkan авто-определение) | `scripts/setup.sh/bat`, `scripts/run.sh/bat` |
| Config (LLM) | ✅ Реализован (model profiles, server args) | `config/llm.toml` |
| ToolModule | ✅ Реализован (file_read/write/run) | `modules/tool_module.rs` |
| AgentModule (State Machine) | **Базовый** → нужна 9-фазная State Machine | `modules/agent_module.rs` |
| ContextCompressor | **Не начат** → Phase 6 | — |
| ImpactAssessment | **Не начат** → Phase 6 | — |
| DocSyncAgent | **Не начат** → Phase 6 | — |
| Agent UI (status, tasks) | **Не начат** → Phase 6 | — |
| Module Compiler | **Не начат** → Phase 7 | — |
| AuditLogger | **Не начат** → Phase 8 | — |

---

## Приоритет реализации

```
Фаза 1-5 ✅ ──► Фаза 6 (Agent) ──► Фаза 7 (Self-Improvement) ──► Фаза 8 (Security) ──► Фаза 9 (Release) ──► Фаза 10 (Butler)
                     │                      │                          │
                State Machine (9 фаз)   Module Compiler            Audit log
                Tool Use Protocol       WASM modules               RBAC
                ImpactAssessment        Versioning                 Sandboxed exec
                ContextCompressor       Rollback
                DocSyncAgent
                Agent UI
```

Фазы 1-5 завершены. Фаза 6 (Autonomous Agent) — следующий шаг и фундамент автономности.

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
