# ISKIN CLI - Архитектура

## Видение

**ISKIN CLI** — автономный AI-агент уровня Devin, работающий в терминале.

```
┌─────────────────────────────────────────────────────────┐
│                      CLI (терминал)                      │
│  $ iskin "создай файл hello.py"                         │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              OpenHands SDK (Python)                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐   │
│  │ Agent       │ │ Tools       │ │ Conversation    │   │
│  │ (LLM loop)  │ │(file/shell) │ │ (history)      │   │
│  └─────────────┘ └─────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│               LLM (локальный или облачный)               │
│  ┌──────────────────────────────────────────────┐      │
│  │ llama.cpp (Qwen2.5-Coder) или облако         │      │
│  └──────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────┘
```

## Компоненты

### iskin_cli/ — CLI обёртка

```
iskin_cli/
├── src/iskin/
│   ├── cli.py           # Main entry
│   └── tools/           # Custom tools
│       ├── memory.py    # KnowledgeBase, VectorSearch
│       └── sandbox.py   # Docker, HealthCheck
├── examples/
│   └── 01_basic.py     # Пример использования
└── pyproject.toml      # Dependencies
```

### openhands-sdk/ — Agent Framework

- `Agent` — Reasoning loop
- `LLM` — Language model interface
- `Conversation` — State management
- `Tool` — Action/Observation pattern
- `Skills` — Reusable prompts

### openhands-tools/ — Built-in Tools

- `TerminalTool` — Shell commands
- `FileEditorTool` — file_editor operations
- `GrepTool` — Code search
- `TaskTrackerTool` — Task management

### openhands-workspace/ — Workspace

- `LocalWorkspace` — Local filesystem
- `DockerWorkspace` — Isolated containers

## Поток данных

```
User Input (CLI)
    ↓
cli.py (argparse)
    ↓
Conversation.send_message()
    ↓
Agent.reasoning_loop()
    ↓
LLM.predict()
    ↓
Tool.execute()
    ↓
Observation → User
```

## Конфигурация

### config/llm.toml

```toml
[llm]
endpoint = "http://localhost:8080"  # или облако
model = "qwen2.5-coder"
context_length = 8192
```

## Зависимости

- Python 3.10+
- openhands-sdk
- openhands-tools

## Сборка

```bash
cd iskin_cli
pip install -e .
```
