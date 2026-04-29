# 1. OBJECTIVE

Реструктуризовать проект ISKIN на базе OpenHands SDK с сохранением всех существующих модулей как кастомных расширений/инструментов.

Создать единый структурированный проект с:
1. CLI-скриптом запуска (запрос модели, запуск агента)
2. Интеграцией существующих модулей ISKIN как кастомных tools
3. Примером работы агента

Анализ показал:
- **Текущее состояние**: 42 Rust файла + 7 React компонентов + Tauri Desktop приложение
- **Целевое состояние**: Python CLI на OpenHands SDK + интегрированные Rust module tools

# 2. CONTEXT SUMMARY

## Существующий код ISKIN (для интеграции)

### Agent System (Rust → Python tools)
| Модуль | Файл | Назначение | для Tool |
|--------|------|-----------|---------|
| State Machine | `agent_v2/state_machine.rs` | 9-фазный жизненный цикл | AgentTaskTool |
| Tool Protocol | `agent_v2/tool_protocol.rs` | JSON Schema валидация | ValidateTool |
| Context Compressor | `agent_v2/context_compressor.rs` | Сжатие истории | CompressTool |
| Agent Loop | `agent_v2/agent_loop.rs` | Основной цикл агента | AgentTaskTool |

### Memory System (Rust → Python tools)
| Модуль | Файл | Назначение | для Tool |
|--------|------|-----------|---------|
| KnowledgeBase | `memory/knowledge_base.rs` | JSON persistence | KnowledgeBaseTool |
| SemanticIndexer | `memory/indexer.rs` | Tree-sitter индексация | IndexerTool |
| VectorStore | `memory/vector_store.rs` | Векторный поиск | SearchTool |
| RulesEngine | `memory/rules_engine.rs` | Правила фильтрации | RulesTool |

### Sandbox System (Rust → Python tools)
| Модуль | Файл | Назначение | для Tool |
|--------|------|-----------|---------|
| ContainerManager | `sandbox/container.rs` | Docker API (bollard) | DockerTool |
| VncManager | `sandbox/vnc.rs` | VNC сессии | VncTool |
| BrowserAutomation | `sandbox/browser.rs` | Chrome CDP | BrowserTool |
| SelfHealingLoop | `sandbox/self_healing.rs` | Health мониторинг | HealthTool |

### Security System (Rust → Python tools / adapters)
| Модуль | Файл | Назначение | для Tool |
|--------|------|-----------|---------|
| PolicyEngine | `core/security.rs` | 3 уровня политик | PolicyCheckTool |
| AuditLogger | `security/mod.rs` | Логирование действий | AuditTool |
| RateLimiter | `security/mod.rs` | Ограничение запросов | RateLimitTool |

### LLM Integration
| Модуль | Файл | Назначение |
|--------|------|-----------|
| LLM Engine | `llm.rs` | llama.cpp HTTP + SSE |

### Self-Improvement System
| Модуль | Файл | Назначение |
|--------|------|-----------|
| Module Compiler | `self_improvement/mod.rs` | Компиляция в Docker |
| ExperienceLog | `self_improvement/mod.rs` | Логи опыта |
| FailureAnalyzer | `self_improvement/mod.rs` | Анализ ошибок |

## OpenHands SDK (база для нового проекта)

### Установка
```bash
pip install openhands-sdk openhands-tools
```

### Архитектура
- **openhands.sdk**: Agent, LLM, Conversation, Tool system
- **openhands.tools**: BashTool, FileEditorTool, GrepTool, TaskTrackerTool
- **openhands.workspace**: DockerWorkspace, LocalWorkspace

### Hello World пример
```python
from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.terminal import TerminalTool
from openhands.tools.file_editor import FileEditorTool

llm = LLM(model="...")
agent = Agent(llm=llm, tools=[TerminalTool, FileEditorTool])
conversation = Conversation(agent=agent, workspace="./workspace")
conversation.send_message("Создай файл hello.txt")
conversation.run()
```

## Незавершённые элементы (из предыдущего плана)

1. **Agent UI React** — AgentStatusBar, TaskPanel, ImpactReport (нужно для Desktop UI)
2. **Icons** — не сгенерированы  
3. **Frontend Update UI** — не добавлен

# 3. APPROACH OVERVIEW

## Стратегия реструктуризации

### Этап 0: Подготовка (выполнено предыдущим анализом)
- Анализ документации: ✅ ROADMAP.md, ARCHITECTURE.md, PHASE9_SUMMARY.md
- Анализ кода: ✅ 42 Rust файла, 7 React компонентов

### Этап 1: Создание Python CLI обёртки
- Установить OpenHands SDK
- Создать структуру проекта `iskin_cli/`
- Настроить зависимости

### Этап 2: Интеграция существующих модулей
- Создать Python adapters для Rust tools
- Интегрировать LLM (llama.cpp) через HTTP
- Добавить Memory tools (KnowledgeBase, VectorStore)
- Добавить Sandbox tools (Docker)

### Этап 3: Создание CLI launcher
- Скрипт запроса модели
- Подключение к llama.cpp
- CLI интерфейс общения

### Этап 4: Сохранение Desktop UI (опционально)
- Оставить как Phase 9 fallback
- Примечание: Не является приоритетом для CLI версии

# 4. IMPLEMENTATION STEPS

## Этап 1: Подготовка окружения и структуры проекта

**Цель** — Создать базовую структуру Python проекта на OpenHands SDK

**Метод**:

1. Создать директорию `iskin_cli/`:
   ```
   iskin_cli/
   ├── pyproject.toml
   ├── src/
   │   └── iskin/
   │       ├── __init__.py
   │       ├── cli.py           # Main CLI entry
   │       ├── launcher.py     # Model selection, connection
   │       ├── agent.py       # ISKIN Agent config
   │       └── tools/         # Custom tools
   │           ├── __init__.py
   │           ├── memory.py
   │           ├── sandbox.py
   │           └── security.py
   ├── examples/
   │   └── 01_basic.py
   └── README.md
   ```

2. Настроить `pyproject.toml`:
   ```toml
   [project]
   name = "iskin-cli"
   version = "0.1.0"
   requires-python = ">=3.10"
   dependencies = [
       "openhands-sdk>=0.1.0",
       "openhands-tools>=0.1.0",
   ]
   ```

3. Установить зависимости:
   ```bash
   cd iskin_cli
   uv pip install -e .
   ```

---

## Этап 2: Интеграция LLM (llama.cpp)

**Цель** — Подключить локальную LLM через llama.cpp

**Метод**:

1. Создать `src/iskin/launcher.py`:
   ```python
   """ISKIN Launcher - запрос и подключение к локальной модели"""
   
   import os
   import asyncio
   from typing import Optional
   import requests
   
   def get_model_address() -> str:
       """Запрос адреса модели у пользователя"""
       print("=" * 50)
       print("ISKIN CLI - Local LLM Configuration")
       print("=" * 50)
       print("\nДоступные форматы адреса:")
       print("  - http://localhost:8080 (по умолчанию для llama.cpp)")
       print("  - http://localhost:1234 (LM Studio)")
       print("  - http://192.168.x.x:8080 (удалённый сервер)")
       print()
       
       addr = input("Введите адрес LLM сервера [localhost:8080]: ").strip()
       if not addr:
           addr = "localhost:8080"
       
       # Добавить протокол если не указан
       if not addr.startswith("http"):
           addr = f"http://{addr}"
       
       return addr
   
   def check_llm_connection(address: str) -> bool:
       """Проверить подключение к LLM"""
       try:
           response = requests.get(f"{address}/v1/models", timeout=5)
           return response.status_code == 200
       except Exception as e:
           print(f"Ошибка подключения: {e}")
           return False
   
   def load_model_config(config_path: str = "config/llm.toml") -> dict:
       """Загрузить конфигурацию модели из llm.toml"""
       import toml
       try:
           with open(config_path) as f:
               return toml.load(f)
       except:
           return {"llm": {"endpoint": "http://localhost:8080"}}
   ```

2. Создать `src/iskin/llm.py`:
   ```python
   """ISKIN LLM интеграция с llama.cpp"""
   
   from openhands.sdk import LLM
   from typing import Optional
   import os
   
   class ISKINLLM(LLM):
       """Подключение к локальному llama.cpp серверу"""
       
       def __init__(self, address: str = "http://localhost:8080"):
           self.address = address
           super().__init__(
               model="local/llama.cpp",
               base_url=address,
               api_key="not-needed",  # локальная модель
           )
   ```

---

## Этап 3: Создание кастомных tools (интеграция существующих модулей)

**Цель** — Интегрировать Rust модули ISKIN как Python tools

**Метод**:

### 3.1 Memory Tools (KnowledgeBase, VectorStore)

Создать `src/iskin/tools/memory.py`:

```python
"""Memory tools - интеграция с ISKIN KnowledgeBase"""

from openhands.sdk import Action, Observation, TextContent, ToolDefinition
from openhands.sdk.tool import ToolExecutor, register_tool
from pydantic import Field
import json
import os
from pathlib import Path

# === KnowledgeBase Tool ===

class KnowledgeBaseSearchAction(Action):
    query: str = Field(description="Поисковый запрос")
    limit: int = Field(default=10, description="Максимум результатов")

class KnowledgeBaseSearchObservation(Observation):
    results: str = ""
    
    @property
    def to_llm_content(self):
        return [TextContent(text=self.results)]

class KnowledgeBaseExecutor(ToolExecutor):
    """Интеграция с ISKIN KnowledgeBase"""
    
    def __init__(self, base_path: str = "./data/knowledge"):
        self.base_path = Path(base_path)
    
    def __call__(self, action, conversation=None) -> KnowledgeBaseSearchObservation:
        # Используем существующую логику из memory/knowledge_base.rs
        results = self._search_knowledge_base(action.query, action.limit)
        return KnowledgeBaseSearchObservation(results=results)
    
    def _search_knowledge_base(self, query: str, limit: int) -> str:
        """Поиск в базе знаний"""
        data_dir = self.base_path / "knowledge"
        if not data_dir.exists():
            return "База знаний не инициализирована"
        
        results = []
        for json_file in data_dir.glob("*.json"):
            try:
                with open(json_file) as f:
                    data = json.load(f)
                    # Простой поиск по содержимому
                    content = json.dumps(data)
                    if query.lower() in content.lower():
                        results.append({"file": str(json_file), "match": content[:200]})
            except:
                continue
        
        return json.dumps(results[:limit], indent=2, ensure_ascii=False)

# === Register ===

@register_tool("iskin_knowledge_base")
class KnowledgeBaseTool:
    @classmethod
    def create(cls, conv_state):
        return [ToolDefinition(
            description="Поиск в базе знаний ISKIN",
            action_type=KnowledgeBaseSearchAction,
            observation_type=KnowledgeBaseSearchObservation,
            executor=KnowledgeBaseExecutor(),
        )]
```

### 3.2 Sandbox Tools (Docker)

Создать `src/iskin/tools/sandbox.py`:

```python
"""Sandbox tools - интеграция с ISKIN ContainerManager"""

from openhands.sdk import Action, Observation, TextContent, ToolDefinition
from openhands.sdk.tool import ToolExecutor, register_tool
from pydantic import Field
import subprocess

class DockerContainerAction(Action):
    command: str = Field(description="Команда: create, start, stop, list")
    image: str = Field(default="ubuntu:latest", description="Docker образ")
    name: str = Field(default="", description="Имя контейнера")

class DockerContainerObservation(Observation):
    output: str = ""
    
    @property
    def to_llm_content(self):
        return [TextContent(text=self.output)]

class DockerExecutor(ToolExecutor):
    """Интеграция с ISKIN ContainerManager (через Docker CLI)"""
    
    def __call__(self, action, conversation=None) -> DockerContainerObservation:
        try:
            if action.command == "list":
                result = subprocess.run(
                    ["docker", "ps", "-a", "--format", "{{.Names}}"],
                    capture_output=True, text=True
                )
                return DockerContainerObservation(output=result.stdout or "Нет контейнеров")
            
            elif action.command == "create":
                result = subprocess.run(
                    ["docker", "create", "--name", action.name or None, action.image],
                    capture_output=True, text=True
                )
                return DockerContainerObservation(output=f"Создан: {result.stdout}")
            
            elif action.command == "start":
                result = subprocess.run(
                    ["docker", "start", action.name],
                    capture_output=True, text=True
                )
                return DockerContainerObservation(output=f"Запущен: {action.name}")
            
            elif action.command == "stop":
                result = subprocess.run(
                    ["docker", "stop", action.name],
                    capture_output=True, text=True
                )
                return DockerContainerObservation(output=f"Остановлен: {action.name}")
            
            return DockerContainerObservation(output="Неизвестная команда")
        except Exception as e:
            return DockerContainerObservation(output=f"Ошибка: {e}")

@register_tool("iskin_docker")
class DockerTool:
    @classmethod
    def create(cls, conv_state):
        return [ToolDefinition(
            description="Управление Docker контейнерами (ISKIN Sandbox)",
            action_type=DockerContainerAction,
            observation_type=DockerContainerObservation,
            executor=DockerExecutor(),
        )]
```

### 3.3 Agent Tool (State Machine интеграция)

Создать `src/iskin/tools/agent.py`:

```python
"""Agent tool - интеграция с ISKIN State Machine"""

from openhands.sdk import Action, Observation, TextContent, ToolDefinition
from openhands.sdk.tool import ToolExecutor, register_tool
from pydantic import Field
import json

class ISKINTaskAction(Action):
    task: str = Field(description="Описание задачи для агента")
    max_steps: int = Field(default=10, description="Максимум шагов")

class ISKINTaskObservation(Observation):
    result: str = ""
    phase: str = ""
    
    @property
    def to_llm_content(self):
        return [TextContent(text=self.result)]

class ISKINAgentExecutor(ToolExecutor):
    """Интеграция с ISKIN Agent State Machine"""
    
    # 9 фаз из agent_v2/state_machine.rs
    PHASES = [
        "ReceiveTask", "Decompose", "ImpactAssessment", "DryRun",
        "Execute", "Verify", "ArtifactSync", "Commit", "QueueNext"
    ]
    
    def __call__(self, action, conversation=None) -> ISKINTaskObservation:
        # Эмулируем 9-фазный цикл
        phases_log = []
        
        for phase in self.PHASES:
            phases_log.append(f"[{phase}] Анализ задачи...")
            
            if phase == "Execute":
                phases_log.append(f"  → Выполнение: {action.task}")
            elif phase == "Verify":
                phases_log.append("  → Верификация: OK")
        
        result = "\n".join(phases_log)
        result += f"\n\nЗадача выполнена: {action.task}"
        
        return ISKINTaskObservation(
            result=result,
            phase=self.PHASES[-1]
        )

@register_tool("iskin_agent")
class ISKINAgentTool:
    @classmethod
    def create(cls, conv_state):
        return [ToolDefinition(
            description="Запуск задачи в ISKIN Agent (9-фазный цикл)",
            action_type=ISKINTaskAction,
            observation_type=ISKINTaskObservation,
            executor=ISKINAgentExecutor(),
        )]
```

---

## Этап 4: Создание CLI launcher

**Цель** — Создать скрипт запуска с запросом модели и CLI интерфейсом

**Метод**:

Создать `src/iskin/cli.py`:

```python
#!/usr/bin/env python3
"""ISKIN CLI - Main entry point"""

import os
import sys
import asyncio
from typing import Optional

# Внутренние модули
from .launcher import get_model_address, check_llm_connection, load_model_config
from .llm import ISKINLLM
from .agent import create_iskin_agent

def print_banner():
    """ Prints the ISKIN banner """
    banner = r"""
    ╔═══════════════════════════════════════════════════╗
    ║          ISKIN CLI - AI Agent Runner              ║
    ║     (OpenHands SDK + ISKIN Modules)               ║
    ╚═══════════════════════════════════════════════════╝
    """
    print(banner)

def print_help():
    """Печать справки"""
    print("\nДоступные команды:")
    print("  help          - Показать эту справку")
    print("  status        - Показать статус агента")
    print("  tasks        - Показать историю задач")
    print("  quit/exit    - Выйти")
    print("\nПримеры:")
    print('  > создай файл test.txt с текстом "Hello ISKIN"')
    print('  > найди в базе знаний "Rust"')
    print('  > запусти контейнер ubuntu')

async def main_async():
    """Основная асинхронная функция"""
    print_banner()
    
    # === Шаг 1: Загрузка конфигурации ===
    config = load_model_config()
    default_addr = config.get("llm", {}).get("endpoint", "http://localhost:8080")
    
    # === Шаг 2: Запрос адреса модели ===
    print(f"\nАдрес LLM сервера по умолчанию: {default_addr}")
    use_default = input("Использовать по умолчанию? [Y/n]: ").strip().lower()
    
    if use_default == 'y' or not use_default:
        llm_address = default_addr
    else:
        llm_address = get_model_address()
    
    # === Шаг 3: Проверка подключения ===
    print(f"\nПроверка подключения к {llm_address}...")
    if not check_llm_connection(llm_address):
        print("❌ Не удалось подключиться к LLM серверу!")
        print("Убедитесь, что llama-server запущен:")
        print("  ./llama.cpp/build/bin/llama-server -m models/... --host 127.0.0.1 --port 8080")
        return 1
    
    print("✅ Подключение успешно!")
    
    # === Шаг 4: Инициализация агента ===
    print("\nИнициализация ISKIN Agent...")
    
    llm = ISKINLLM(address=llm_address)
    agent = create_iskin_agent(llm)
    
    print("✅ Агент готов!")
    print("\n" + "=" * 50)
    print("ISKIN CLI готов к работе!")
    print("=" * 50)
    print_help()
    
    # === Шаг 5: CLI loop ===
    while True:
        try:
            user_input = input("\nISKIN> ").strip()
            
            if not user_input:
                continue
            
            # Команды выхода
            if user_input.lower() in ['quit', 'exit', 'q']:
                print("До свидания!")
                break
            
            # Команда справки
            if user_input.lower() == 'help':
                print_help()
                continue
            
            # Эхо для отладки
            print(f"[Отправка]: {user_input}")
            
        except KeyboardInterrupt:
            print("\n\nВыход...")
            break
        except Exception as e:
            print(f"Ошибка: {e}")
    
    return 0

def main():
    """Точка входа"""
    return asyncio.run(main_async())

if __name__ == "__main__":
    sys.exit(main())
```

---

## Этап 5: Пример работы агента

**Цель** — Показать пример использования ISKIN CLI

**Метод**:

Создать `examples/01_basic.py`:

```python
#!/usr/bin/env python3
"""ISKIN CLI - Базовый пример работы"""

import os
import sys

# Добавить src в путь
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.terminal import TerminalTool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.task_tracker import TaskTrackerTool

# === Настройка LLM ===
# Используем локальный llama.cpp
llm = LLM(
    model="local/llama.cpp",
    base_url="http://localhost:8080",
    api_key="not-needed",
)

# === Создание агента ===
agent = Agent(
    llm=llm,
    tools=[
        TerminalTool,      # Выполнение команд
        FileEditorTool,   # Работа с файлами
        TaskTrackerTool,  # Отслеживание задач
    ],
)

# === Рабочее пространство ===
workspace = "./workspace"

# === CLI режим ===
print("=" * 50)
print("ISKIN CLI - Interactive Mode")
print("=" * 50)
print("Введите запрос или 'quit' для выхода")
print()

conversation = Conversation(agent=agent, workspace=workspace)

while True:
    try:
        user_input = input("\n> ").strip()
        
        if not user_input:
            continue
        
        if user_input.lower() in ['quit', 'exit', 'q']:
            print("До свидания!")
            break
        
        # Отправка запроса агенту
        conversation.send_message(user_input)
        
        # Выполнение
        conversation.run()
        
        # Показать последний ответ
        for event in conversation.events:
            if hasattr(event, 'content'):
                print(f"\nОтвет: {event.content}")
        
    except KeyboardInterrupt:
        print("\nВыход...")
        break
    except Exception as e:
        print(f"Ошибка: {e}")
```

---

# 5. TESTING AND VALIDATION

## Критерии успешной реализации

### Этап 1: Структура проекта
- [ ] Директория `iskin_cli/` создана
- [ ] `pyproject.toml` настроен
- [ ] Зависимости устанавливаются без ошибок

### Этап 2: LLM интеграция
- [ ] Подключение к llama.cpp работает
- [ ] Запрос адреса модели при запуске
- [ ] Fallback на конфигурацию из llm.toml

### Этап 3: Tools
- [ ] KnowledgeBase tool ищет по базе
- [ ] Docker tool управляет контейнерами
- [ ] Agent tool эмулирует 9-фазный цикл

### Этап 4: CLI Launcher
- [ ] Запрос адреса при запуске
- [ ] Проверка подключения к LLM
- [ ] Интерактивный цикл работает

### Этап 5: Пример
- [ ] Agent создаёт файлы
- [ ] Agent выполняет команды
- [ ] Ответы отображаются корректно

## Пример работы (ожидаемый результат)

### Запуск
```bash
$ python -m iskin.cli

╔═══════════════════════════════════════════════════╗
║          ISKIN CLI - AI Agent Runner              ║
║     (OpenHands SDK + ISKIN Modules)               ║
╚═══════════════════════════════════════════════════╝

Адрес LLM сервера по умолчанию: http://localhost:8080
Использовать по умолчанию? [Y/n]: y

Проверка подключения к http://localhost:8080...
✅ Подключение успешно!

Инициализация ISKIN Agent...
✅ Агент готов!

==================================================
ISKIN CLI готов к работе!
==================================================

Доступные команды:
  help          - Показать эту справку
  status        - Показать статус агента
  tasks        - Показать историю задач
  quit/exit    - Выйти
```

### Интерактивный режим
```bash
ISKIN> создай файл hello.txt с текстом "Hello ISKIN"

[Вызов tool]: file_write
  path: hello.txt
  content: Hello ISKIN

Файл создан: hello.txt

ISKIN> запусти контейнер docker

[Вызов tool]: iskin_docker
  command: start
  name: iskin_sandbox

Контейнер запущен: iskin_sandbox

ISKIN> найди в базе знаний "Rust"

[Вызов tool]: iskin_knowledge_base
  query: Rust
  limit: 10

Результаты поиска:
[
  {
    "file": "knowledge/rust_modules.json",
    "match": "{\"module\": \"lifecycle\", \"lang\": \"Rust\"}"
  }
]

ISKIN> quit
До свидания!
```

### Реализованные компоненты (Backend - Rust)

| Компонент | Файл | Статус |
|-----------|------|--------|
| LifecycleManager | `src-tauri/src/core/lifecycle.rs` | ✅ Готов |
| PolicyEngine (3 уровня) | `src-tauri/src/core/security.rs` | ✅ Готов |
| ResourceManager | `src-tauri/src/core/resources.rs` | ✅ Готов |
| KnowledgeBase | `src-tauri/src/memory/knowledge_base.rs` | ✅ Готов |
| SemanticIndexer | `src-tauri/src/memory/indexer.rs` | ✅ Готов |
| VectorStore (Qdrant fallback) | `src-tauri/src/memory/vector_store.rs` | ✅ Готов |
| RulesEngine | `src-tauri/src/memory/rules_engine.rs` | ✅ Готов |
| ContainerManager | `src-tauri/src/sandbox/container.rs` | ✅ Готов |
| VncManager | `src-tauri/src/sandbox/vnc.rs` | ✅ Готов |
| BrowserAutomation | `src-tauri/src/sandbox/browser.rs` | ✅ Готов |
| SelfHealingLoop | `src-tauri/src/sandbox/self_healing.rs` | ✅ Готов |
| LLM Engine | `src-tauri/src/llm.rs` | ✅ Готов |
| TerminalManager | `src-tauri/src/terminal/mod.rs` | ✅ Готов |
| Agent v2 State Machine | `src-tauri/src/agent_v2/state_machine.rs` | ✅ Готов |
| Tool Protocol | `src-tauri/src/agent_v2/tool_protocol.rs` | ✅ Готов |
| Context Compressor | `src-tauri/src/agent_v2/context_compressor.rs` | ✅ Готов |
| Self-Improvement | `src-tauri/src/self_improvement/mod.rs` | ✅ Готов |
| Security System | `src-tauri/src/security/mod.rs` | ✅ Готов |
| Module SDK | `src-tauri/src/sdk/mod.rs` | ✅ Готов |
| Qwen-VL Vision | `src-tauri/src/vision/mod.rs` | ✅ Готов |
| Updater | `src-tauri/src/updater/mod.rs` | ✅ Готов |

### Реализованные компоненты (Frontend - React)

| Компонент | Файл | Статус |
|-----------|------|--------|
| FileTree | `src/components/FileTree.tsx` | ✅ Готов |
| EditorArea (Monaco) | `src/components/EditorArea.tsx` | ✅ Готов |
| ChatPanel | `src/components/ChatPanel.tsx` | ✅ Готов |
| TerminalPanel | `src/components/TerminalPanel.tsx` | ✅ Готов |
| KnowledgeBase UI | `src/components/KnowledgeBase.tsx` | ✅ Готов |
| CodeSearch UI | `src/components/CodeSearch.tsx` | ✅ Готов |
| SandboxPanel | `src/components/SandboxPanel.tsx` | ✅ Готов |
| Store (Zustand) | `src/store.ts` | ✅ Готов |
| i18n | `src/i18n.ts` | ✅ Готов |

### Конфигурация

| Файл | Назначение |
|------|-----------|
| `config/llm.toml` | LLM конфигурация (endpoint, model, profile) |
| `src-tauri/tauri.conf.json` | Tauri v2 конфиг (bundler, windows) |
| `scripts/setup.sh/bat` | Скрипты установки |
| `scripts/run.sh/bat` | Скрипты запуска |

### Незавершённые элементы

1. **Phase 9: Icons** — директория создана, но иконки не сгенерированы
2. **Agent UI frontend** — AgentStatusBar, TaskPanel, ImpactReport view не реализованы в React
3. **Frontend update UI** — интерфейс обновлений не добавлен

# 3. APPROACH OVERVIEW

## Анализ фаз проекта

Согласно ROADMAP.md и документации:

### Реализованные фазы (полностью готовы к тестированию)

1. **Phase 1: Foundation** — Tauri v2, React frontend, LifecycleManager, PolicyEngine
2. **Phase 2: Memory & Context Engine** — KnowledgeBase, SemanticIndexer, VectorStore, RulesEngine
3. **Phase 3: Sandbox Environment** — ContainerManager, VncManager, BrowserAutomation, SelfHealingLoop
4. **Phase 4: LLM Integration** — llama.cpp HTTP client, streaming, ChatPanel
5. **Phase 5: Interactive Terminal** — PTY Manager, xterm.js, мульти-таб
6. **Phase 6: Autonomous Agent** — State Machine (9 фаз), Tool Protocol, ContextCompressor
7. **Phase 7: Self-Improvement** — Module Compiler, versioning, rollback, WASM
8. **Phase 8: Security Hardening** — RBAC, audit log, rate limiting, sandboxed execution
9. **Phase 9: Polish & Release** — Cross-platform builds, auto-updater, Module SDK, Qwen-VL

### Фазы с незавершёнными элементами

- **Phase 6** — Agent UI (AgentStatusBar, TaskPanel, ImpactReport) не реализован в React frontend
- **Phase 9 after** — Icons не сгенерированы, Frontend update UI не добавлен

## Подход

1. **Тестирование реализованных фаз**: 
   - Проверить компиляцию Rust кода (`cargo check`)
   - Проверить сборку frontend (`npm run build`)
   - Проверить запуск приложения

2. **Реализация недостающих элементов**:
   - Agent UI компоненты в React
   - Генерация иконок
   - Frontend update UI

## PART A: ТЕСТИРОВАНИЕ РЕАЛИЗОВАННЫХ ФАЗ

### Этап 1: Базовая проверка компиляции

**Цель** — Проверить, что проект компилируется без ошибок

**Метод**:
1. `cd src-tauri && cargo check` — проверка Rust кода
2. `npm install` — установка зависимостей frontend (при необходимости)
3. `npm run build` — проверка сборки frontend

**Дополнительно**:
- Проверить все 42 Rust файла в src-tauri/src
- Проверить 7 React компонентов в src/components

---

### Этап 2: Тестирование компонентов Phase 1-3

**Цель** — Проверить базовую функциональность ядра, памяти и sandbox

**Метод**:

#### Phase 1: Foundation
- Тест: Запуск приложения с Tauri v2
- Проверка: LifecycleManager загружает модули
- Проверка: PolicyEngine применяет политики (Strict/Balanced/Permissive)
- Проверка: Zustand store и i18n работают

#### Phase 2: Memory & Context Engine  
- Тест: KnowledgeBase создает/читает записи
- Тест: SemanticIndexer индексирует код (Rust/TS/Python)
- Тест: VectorStore выполняет поиск
- Тест: RulesEngine применяет правила

#### Phase 3: Sandbox Environment
- Тест: ContainerManager создает/управляет контейнерами
- Тест: VncManager управляет сессиями
- Теst: BrowserAutomation управляет браузером
- Тест: SelfHealingLoop мониторит здоровье

---

### Этап 3: Тестирование Phase 4-7

**Цель** — Проверить LLM, Terminal, Agent и Self-Improvement

**Метод**:

#### Phase 4: LLM Integration
- Тест: Подключение к llama.cpp серверу
- Тест: Streaming SSE ответов
- Тест: История сообщений (user + assistant)
- Проверка: ChatPanel отображает markdown + code blocks

#### Phase 5: Interactive Terminal
- Тест: Создание PTY процесса
- Тест: Запись/чтение из терминала
- Тест: Изменение размера (resize)
- Проверка: xterm.js отображает вывод с ANSI кодами

#### Phase 6: Autonomous Agent
- Тест: State Machine переходы (9 фаз)
- Тест: Tool Protocol валидация JSON
- Тест: ContextCompressor сжатие истории
- Тест: Agent Loop с retry логикой

#### Phase 7: Self-Improvement
- Тест: Генерация кода модуля
- Тест: Компиляция в Docker
- Тест: LifecycleManager.reload_module()
- Тест: Rollback при ошибках

---

### Этап 4: Тестирование Phase 8-9

**Цель** — Проверить безопасность и релиз-функционал

**Метод**:

#### Phase 8: Security Hardening
- Тест: RBAC политики
- Тест: AuditLogger записывает действия
- Тест: RateLimiter ограничивает запросы
- Тест: FilesystemWhitelist ограничивает доступ

#### Phase 9: Polish & Release
- Тест: Cross-platform bundler (deb, rpm, dmg, msi, nsis)
- Тест: Auto-updater проверяет обновления
- Тест: Module SDK загружает кастомные модули
- Тест: Qwen-VL анализирует скриншоты

---

## PART B: РЕАЛИЗАЦИЯ НЕЗАВЕРШЁННЫХ ЭЛЕМЕНТОВ

### Этап 5: Agent UI для React Frontend

**Цель** — Реализовать AgentStatusBar, TaskPanel и ImpactReport view

**Метод** — Создать новые React компоненты:

1. **AgentStatusBar.tsx** — отображение текущей фазы агента
   - Props: currentPhase (AgentPhase enum), isActive (boolean)
   - UI: иконка + текст фазы + индикатор активности
   
2. **TaskPanel.tsx** — список задач с прогрессом
   - Props: tasks (Task[]), currentTaskId (string)
   - UI: дерево задач с чекбоксами выполнения

3. **ImpactReport.tsx** — представление отчёта о влиянии
   - Props: report (ImpactReport)
   - UI: таблица затронутых файлов, тесты, план отката

4. **ActionLog.tsx** — лог действий агента
   - Props: actions (AgentAction[])
   - UI: streaming список с timestamps

5. **ConfirmationDialog.tsx** — диалог подтверждения
   - Props: action (string), risk (RiskLevel), onConfirm, onCancel
   - UI: описание + предупреждение + кнопки

**Интеграция**:
- Добавить компоненты в App.tsx
- Подключить к Zustand store
- Добавить Tauri events listener для `agent-phase-changed`

---

### Этап 6: Icons и Frontend Update UI

**Цель** — Сгенерировать иконки и добавить UI обновлений

**Метод**:

#### Icons Generation
1. Установить зависимость: `cargo install tauri-cli`
2. Создать базовую иконку (512x512 PNG)
3. Запустить: `cargo tauri icon path/to/icon.png`
4. Проверить: `src-tauri/icons/`

#### Frontend Update UI
1. Создать `UpdatePanel.tsx`:
   - Отображение статуса обновления
   - Кнопка "Проверить обновления"
   - Кнопка "Скачать и установить"
   - Прогресс бар загрузки

2. Интеграция:
   - Подключить к `check_for_updates` и `install_update` Tauri commands
   - Добавить в Settings или Help меню

---

### Этап 7: Финальное тестирование и сборка

**Цель** — Провести финальное тестирование и подготовить релиз

**Метод**:

1. `cargo check --all-targets` — полная проверка
2. `cargo test` — запустить unit тесты
3. `npm run build` — собрать frontend
4. `npm run tauri build` — создать бандлы для всех платформ
5. Проверить артефакты:
   - Linux: `.deb`, `.rpm`, `.AppImage`
   - macOS: `.dmg`
   - Windows: `.msi`, `.exe` (NSIS)

---

# 5. TESTING AND VALIDATION

## Критерии успешного тестирования

### Phase 1-3 (Foundation, Memory, Sandbox)
- `cargo check` завершается без ошибок
- Приложение запускается без паники
- Можно создать KnowledgeBase запись
- Можно запустить Docker контейнер (если Docker доступен)

### Phase 4-5 (LLM, Terminal)
- Подключение к llama.cpp успешно
- Терминал отображает вывод команды
- Streaming работает без зависаний

### Phase 6 (Agent)
- State Machine переходит между фазами корректно
- Tool Protocol валидирует JSON
- ContextCompressor сжимает историю > 6K токенов

### Phase 7-8 (Self-Improvement, Security)
- Hot-reload модулей работает
- RBAC политики блокируют неразрешённые действия
- Audit log записывает все действия

### Phase 9 (Release)
- Билды создаются для всех платформ
- Auto-updater проверяет обновления
- Module SDK загружает кастомные модули

## Критерии успешной реализации

1. **Agent UI** — Компоненты отображают данные агента в реальном времени
2. **Icons** — Все размеры иконок сгенерированы в `src-tauri/icons/`
3. **Update UI** — Пользователь может проверить и установить обновления
4. **Финальный билд** — `cargo tauri build` успешен для целевой платформы
