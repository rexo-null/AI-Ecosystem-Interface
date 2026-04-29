# ISKIN CLI - Intelligent Self-Improving Knowledge Interface Network

Автономный AI-агент уровня Devin, работающий в **терминале** на вашем ПК. CLI-утилита для консоли, которая может планировать, писать код, тестировать в песочнице и **улучшать собственные модули** в runtime.

> Никакого GUI. Никакого IDE. Только терминал и агент.

## Ключевые возможности

- **Автономный агент** — 9 фаз: ReceiveTask → Decompose → ImpactAssessment → DryRun → Execute → Verify → ArtifactSync → Commit → QueueNext
- **LLM на вашем ПК** — Qwen2.5-Coder-14B через llama.cpp (без облака, без обёрток)
- **Hot-Reload модули** — агент улучшает свои Dylib/WASM-модули на лету
- **Песочница** — Docker-контейнеры для безопасного тестирования
- **CLI интерфейс** — сессии, стриминг ответов, dry-run/apply, Ctrl+C стоп
- **Память и контекст** — Knowledge Base, векторный поиск, SQLite сессии
- **Self-Improvement** — анализ ошибок → автоапгрейд промптов и тулов

## Quick Start

### Установка

```bash
chmod +x install.sh
./install.sh
```

Скрипт установит: Rust, llama.cpp, модель Qwen2.5-Coder-14B (GGUF).

### Запуск

```bash
# Запуск llama-server + ISKIN CLI
./run.sh

# Или вручную:
./llama.cpp/build/bin/llama-server -m models/qwen2.5-coder-14b.gguf --port 8080
./iskin
```

### Использование

```bash
# Интерактивный режим
./iskin

# Одиночный запрос
./iskin ask "создай файл test.py с unittest"

# Dry-run (без выполнения)
./iskin ask --dry-run "удалить все .tmp файлы"

# Продолжить сессию
./iskin session restore

# Список тулов
./iskin tools list
```

## Архитектура

```
ISKIN CLI/
├── src/                     # Rust Core
│   ├── agent/               # Agent State Machine (9 фаз)
│   ├── llm.rs               # LLM Client (HTTP)
│   ├── tools/               # Tool Registry, Executor
│   ├── sandbox/             # Docker песочница
│   ├── memory/              # Knowledge Base
│   ├── self_improvement/    # Self-Improvement
│   └── cli/                 # CLI интерфейс
├── config/                   # Конфигурация
├── skills/                   # Agent skills (.md)
├── modules/                  # Hot-reload модули
├── DATA/                     # SQLite базы
├── iskin                     # Бинарник
└── README.md
```

## Технический стек

| Слой | Технология |
|------|-----------|
| CLI | clap |
| Backend | Rust (tokio) |
| LLM | llama.cpp HTTP |
| State | SQLite |
| Sandbox | Docker + bollard |
| Hot-Reload | libloading + wasmtime |

## Agent State Machine

```
ReceiveTask → Decompose → ImpactAssessment → DryRun →
Execute → Verify → ArtifactSync → Commit → QueueNext
```

### Правила переходов

- **DECOMPOSITION_LIMIT**: задача → ≤3 шага за цикл
- **IMPACT_BEFORE_ACTION**: перед Execute обязателен ImpactReport
- **DRY_RUN_FIRST**: рискованные операции → sandbox
- **VERIFY_OR_ROLLBACK**: Verify падает → автооткат
- **ARTIFACT_SYNC_MANDATORY**: после Execute → DocSync

## Self-Improvement

```
1. Агент обнаруживает проблему
2. Генерирует улучшенный код через LLM
3. Компилирует в Docker (cargo build)
4. Тестирует в песочнице
5. PolicyEngine проверяет безопасность
6. Hot-reload модуля
7. Мониторинг, rollback при ошибках
```

## CLI Команды

| Команда | Описание |
|---------|----------|
| `ask <prompt>` | Отправить запрос агенту |
| `ask --dry-run` | Симуляция без выполнения |
| `session list` | Список сессий |
| `session restore` | Восстановить сессию |
| `tools list` | Доступные инструменты |
| `tools add` | Добавить новый инструмент |
| `sandbox exec` | Выполнить в песочнице |
| `kill` | Остановить генерацию |

## Зависимости

```bash
# Linux
apt install pkg-config libssl-dev

# macOS
xcode-select --install

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Лицензия

MIT
