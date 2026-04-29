# ISKIN CLI - Архитектура

## Видение

**ISKIN CLI** — автономный AI-агент уровня Devin, работающий в терминале. Подключается к LLM напрямую (без HTTP).

---

## Архитектура системы

```
┌─────────────────────────────────────────────────────────────────┐
│                        ISKIN CLI                                  │
├─────────────────────────────────────────────────────────────────┤
│  CLI Interface (clap)                                            │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ ask      │ │ session   │ │ tools    │ │ sandbox          │   │
│  │ (prompt)│ │ (SQLite)  │ │(registry│ │ (Docker exec)    │   │
│  └──────────┘ └───────────┘ └──────────┘ └──────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  Agent Core                                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ StateMachine — 9 фаз                                    │    │
│  │ ReceiveTask → Decompose → ImpactAssessment → DryRun →    │    │
│  │ Execute → Verify → ArtifactSync → Commit → QueueNext     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌──────────────────┐ ┌────────────────────┐               │
│  │ ContextManager  │ │ ToolExecutor        │                    │
│  │ (compression)  │ │ (file/shell/search│                    │
│  └──────────────────┘ └────────────────────┘               │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  LLM Engine (прямое подключение)                              │
│  ┌──────────────────────────────────────────────────────┐      │
│  │ llama.cpp (in-process)                                │      │
│  │ llm-core: load_model(), predict(), stream()             │      │
│  │ Нет HTTP сервера, один процесс                    │      │
│  └──────────────────────────────────────────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│  Memory & Storage                                             │
│  ┌──────────────────┐ ┌────────────────────┐               │
│  │ KnowledgeBase   │ │ Sessions (SQLite)   │               │
│  │ (TF-IDF)       │ │                     │               │
│  └──────────────────┘ └────────────────────┘               │
├─────────────────────────────────────────────────────────────────┤
│  Security                                                    │
│  ┌──────────────────┐ ┌────────────────────┐               │
│  │ PolicyEngine    │ │ AuditLogger        │                    │
│  │ (Safe/Confirm│ │                  │                    │
│  │ /Dangerous)  │ │                  │               │
│  └──────────────────┘ └────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Модули

### CLI Interface (`src/cli/`)

| Модуль | Назначение |
|--------|------------|
| `main.rs` | Точка входа, clap команды |
| `commands/ask.rs` | `ask` команда |
| `commands/session.rs` | `session` команда |
| `commands/tools.rs` | `tools` команда |
| `output.rs` | colored output, streaming |

### Agent (`src/agent/`)

| Модуль | Назначение |
|--------|------------|
| `state_machine.rs` | 9 фаз, переходы |
| `planner.rs` | Декомпозиция задач |
| `validator.rs` | Валидация ответов |
| `tool_protocol.rs` | JSON Schema tool calls |
| `context_compressor.rs` | Сжатие контекста |

### LLM (`src/llm/`)

| Модуль | Назначение |
|--------|------------|
| `mod.rs` | LLMEngine |
| `model.rs` | llama.cpp binding |
| `prompt.rs` | System prompts |
| `history.rs` | Conversation history |

### Tools (`src/tools/`)

| Модуль | Назначение |
|--------|------------|
| `registry.rs` | Tool registry |
| `executor.rs` | Tool execution |
| `file_tool.rs` | file_read/write/delete |
| `shell_tool.rs` | shell exec |
| `search_tool.rs` | code search |

### Sandbox (`src/sandbox/`)

| Модуль | Назначение |
|--------|------------|
| `container.rs` | Docker exec |
| `dry_run.rs` | Симуляция |
| `self_healing.rs` | Auto-restart |

### Memory (`src/memory/`)

| Модуль | Назначение |
|--------|------------|
| `knowledge.rs` | Knowledge Base |
| `vector.rs` | Vector store |
| `rules.rs` | Rules engine |

### Self-Improvement (`src/self_improvement/`)

| Модуль | Назначение |
|--------|------------|
| `experience.rs` | Experience log |
| `analyzer.rs` | Failure analyzer |
| `improver.rs` | Self-improver |

---

## Прямое подключение к LLM

### Текущая архитектура (HTTP)

```
CLI → HTTP request → llama-server → model
```

### Новая архитектура (in-process)

```
CLI → llm-core → model (один процесс)
```

### Преимущества

| Метрика | HTTP | In-process |
|---------|------|------------|
| Задержка | ~50ms | ~0ms |
| Память | 2x (server + cli) | 1x |
| Control | Ограничен | Полный |
| Streaming | SSE | Прямой |

### Реализация

```rust
// Прямое подключение
use llama_core::Model;

let model = Model::load("qwen2.5-coder-14b.gguf")?;
let mut session = model.create_session();

// Стриминг
while let Some(token) = session.predict("prompt")? {
    print!("{}", token);
    stdout.flush()?;
}
```

---

## Agent State Machine

### Фазы

```
ReceiveTask → Decompose → ImpactAssessment → DryRun →
Execute → Verify → ArtifactSync → Commit → QueueNext
```

### Правила

- **DECOMPOSITION_LIMIT**: ≤3 шага за цикл
- **IMPACT_BEFORE_ACTION**: перед Execute → ImpactReport
- **DRY_RUN_FIRST**: рискованные → sandbox
- **VERIFY_OR_ROLLBACK**: Verify падает → rollback
- **ARTIFACT_SYNC_MANDATORY**: после Execute → DocSync

---

## Конфигурация

### config/llm.toml

```toml
[llm]
model_path = "models/qwen2.5-coder-14b.gguf"
context_length = 8192
temperature = 0.2
gpu_layers = 45
threads = 8

[server]
# Не нужен при in-process
# host = "127.0.0.1"
# port = 8080
```

### config/policy.toml

```toml
[policy]
default = "confirm"

[levels]
safe = ["file_read", "search"]
confirm = ["file_write", "shell_exec"]
dangerous = ["file_delete", "sudo", "module_reload"]
```

---

## Сессии

```bash
# Создать сессию
./iskin ask "создать проект"

# Сохраняется в SQLite
./iskin session list

# Восстановить
./iskin session restore <id>
```

---

## Зависимости

```toml
[dependencies]
clap = "4.5"
tokio = { version = "1.0", features = ["rt", "macros"] }
rusqlite = "0.31"
llama-core = { git = "https://github.com/ggerganov/llama.cpp" }
serde = { version = "1.0", features = ["derive"] }
 bollard = "0.16"
```

---

## Сборка

```bash
# Debug
cargo build

# Release
cargo build --release

# Cross-platform
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```
