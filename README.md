# ISKIN CLI

Автономный AI-агент уровня Devin для терминала. Python CLI на базе OpenHands SDK.

> Никакого GUI. Никакого браузера. Только терминал и агент.

## Быстрый старт

### Установка

```bash
# Клонировать репозиторий
git clone https://github.com/rexo-null/AI-Ecosystem-Interface.git
cd AI-Ecosystem-Interface

# Установить зависимости
pip install openhands-sdk openhands-tools

# Или через uv
uv pip install openhands-sdk openhands-tools
```

### Запуск

```bash
# Интерактивный режим
python iskin_cli/src/iskin/cli.py --interactive

# Одиночный запрос
python iskin_cli/src/iskin/cli.py "создай файл hello.py"

# Dry-run (симуляция)
python iskin_cli/src/iskin/cli.py --dry-run "удалить все .tmp"
```

## Структура проекта

```
AI-Ecosystem-Interface/
├── iskin_cli/             # Python CLI обёртка
│   ├── src/iskin/cli.py   # Main entry point
│   └── src/iskin/tools/  # Custom tools (memory, sandbox)
├── openhands-sdk/         # OpenHands SDK (Agent framework)
├── openhands-tools/      # Built-in tools
├── openhands-workspace/ # Workspace implementations
└── iskin/docs/         # Документация
```

## Использование

### Python API

```python
from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.preset.default import get_default_tools

llm = LLM(model="local/llama.cpp", base_url="http://localhost:8080")
agent = Agent(llm=llm, tools=get_default_tools())

conversation = Conversation(agent=agent, workspace="./workspace")
conversation.send_message("Создай файл test.py")
conversation.run()
```

### Конфигурация LLM

В `config/llm.toml`:

```toml
[llm]
endpoint = "http://localhost:8080"
model = "qwen2.5-coder"
context_length = 8192
```

## Инструменты

| Инструмент | Описание |
|-----------|---------|
| TerminalTool | Выполнение команд в терминале |
| FileEditorTool | Редактирование файлов |
| GrepTool | Поиск по коду |
| BashTool | Bash команды |

## Кастомизация

Добавь свои tools в `iskin_cli/src/iskin/tools/`:

```python
from openhands.sdk import Action, Observation, ToolDefinition
from openhands.sdk.tool import ToolExecutor

class MyAction(Action):
    query: str

class MyObservation(Observation):
    result: str

class MyExecutor(ToolExecutor[MyAction, MyObservation]):
    def __call__(self, action, conversation=None):
        return MyObservation(result="done")

def my_tool():
    return ToolDefinition(
        name="my_tool",
        action_type=MyAction,
        observation_type=MyObservation,
        executor=MyExecutor(),
    )
```

## Разработка

```bash
# Установить в dev режим
cd iskin_cli
pip install -e .

# Запустить тесты
pytest

# Пример
python examples/01_basic.py
```

## Зависимости

- Python 3.10+
- openhands-sdk
- openhands-tools

Для LLM локально:
- llama.cpp сервер (опционально)
- Модель (Qwen2.5-Coder рекомендуется)

## License

MIT
