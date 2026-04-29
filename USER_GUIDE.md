# ISKIN User Guide

## Introduction

ISKIN — это автономный AI-агент уровня Devin, работающий локально на вашем ПК. Он помогает в разработке кода, управлении проектами, автоматизации задач и самообучении.

### Основные возможности

- **Автономное выполнение задач**: Планирование, анализ, кодирование, тестирование
- **Горячая перезагрузка модулей**: Улучшение собственных модулей без перезапуска
- **Песочница безопасности**: Все опасные операции выполняются в изолированной среде
- **Векторный поиск**: Семантический поиск по коду и документации
- **Интеграция с LLM**: Поддержка llama.cpp, Qwen2.5-Coder и других моделей
- **Мультитерминал**: Несколько PTY-терминалов с xterm.js
- **Управление контейнерами**: Docker-контейнеры, VNC, браузерная автоматизация

---

## Installation

### System Requirements

- **OS**: Linux (Ubuntu 20.04+), macOS 12+, Windows 10+
- **RAM**: Минимум 8GB (рекомендуется 16GB+)
- **Storage**: 10GB свободного места
- **GPU**: Опционально (для ускорения LLM через ROCm/Vulkan/CUDA)

### Quick Install

#### Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/rexo-null/AI-Ecosystem-Interface/main/install.sh | bash
```

#### Windows

```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/rexo-null/AI-Ecosystem-Interface/main/install.bat" -OutFile install.bat
.\install.bat
```

### Manual Installation

1. **Clone repository**:
   ```bash
   git clone https://github.com/rexo-null/AI-Ecosystem-Interface.git
   cd AI-Ecosystem-Interface
   ```

2. **Install dependencies**:
   ```bash
   # Rust (если не установлен)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Node.js 18+
   npm install
   
   # Tauri CLI
   cargo install tauri-cli
   ```

3. **Setup LLM**:
   ```bash
   ./scripts/setup.sh  # Linux/macOS
   scripts\setup.bat   # Windows
   ```

4. **Run application**:
   ```bash
   ./scripts/run.sh    # Linux/macOS
   scripts\run.bat     # Windows
   ```

---

## Configuration

### LLM Configuration (`config/llm.toml`)

```toml
[llm]
model_path = "/path/to/model.gguf"
host = "localhost"
port = 8080
context_length = 8192
gpu_layers = 35

[model_profile]
name = "Qwen2.5-Coder-14B"
type = "code_completion"
max_tokens = 4096
temperature = 0.7

[server_args]
flash_attn = true
cache_reuse = 100
```

### Policy Configuration

Политики безопасности настраиваются через `PolicyEngine`:

- **Safe**: Чтение файлов, поиск кода
- **Confirm**: Запись файлов, выполнение команд
- **Dangerous**: Удаление файлов, sudo, network operations

---

## User Interface

### Main Layout

ISKIN использует интерфейс в стиле VS Code:

1. **Sidebar** (слева):
   - Files: Дерево проекта
   - Knowledge: Векторная база знаний
   - Search: Семантический поиск
   - Sandbox: Управление контейнерами

2. **Editor** (центр):
   - Редактор кода с подсветкой синтаксиса
   - Просмотр файлов

3. **ChatPanel** (справа):
   - Диалог с AI-агентом
   - Streaming ответов
   - Кнопка "Выполнить" для code blocks

4. **Terminal** (внизу):
   - Мультитерминал с вкладками
   - Полноценный PTY

5. **AgentStatusBar** (верхняя панель):
   - Текущее состояние агента
   - Прогресс выполнения задач

### Agent Status Phases

Агент проходит через 9 фаз выполнения задачи:

1. **ReceiveTask**: Получение задачи от пользователя
2. **Decompose**: Разбиение на подзадачи (≤3 за цикл)
3. **ImpactAssessment**: Анализ последствий
4. **DryRun**: Тестовое выполнение в песочнице
5. **Execute**: Выполнение действий
6. **Verify**: Проверка результатов
7. **ArtifactSync**: Синхронизация документации
8. **Commit**: Фиксация изменений
9. **QueueNext**: Переход к следующей задаче

---

## Usage Scenarios

### Scenario 1: Code Generation

1. Откройте ChatPanel
2. Введите запрос: "Создай функцию для парсинга JSON"
3. Агент сгенерирует код
4. Нажмите "Выполнить" для сохранения файла
5. Агент автоматически обновит документацию

### Scenario 2: Bug Fixing

1. Выделите проблемный код в редакторе
2. Спросите агента: "Найди ошибку в этом коде"
3. Агент проанализирует код через ImpactAssessment
4. Выполнит DryRun в песочнице
5. Предложит исправление с Verify

### Scenario 3: Project Refactoring

1. Дайте команду: "Рефакторинг модуля X"
2. Агент разобьет задачу на подзадачи
3. Последовательно выполнит каждую фазу
4. Обновит документацию через ArtifactSync

### Scenario 4: Self-Improvement

Агент может улучшать собственные модули:

1. Обнаруживает медленную работу инструмента
2. Анализирует через FailureAnalyzer
3. Генерирует оптимизированный код
4. Компилирует в Docker-песочнице
5. Выполняет горячую перезагрузку через LifecycleManager

---

## Commands Reference

### Tauri Commands

| Command | Description |
|---------|-------------|
| `llm_chat` | Отправить сообщение LLM |
| `llm_stop_generation` | Остановить генерацию |
| `terminal_create` | Создать новый терминал |
| `terminal_write` | Ввод в терминал |
| `terminal_resize` | Изменить размер терминала |
| `knowledge_add` | Добавить запись в базу знаний |
| `knowledge_search` | Поиск в базе знаний |
| `code_search` | Семантический поиск кода |
| `container_start` | Запустить контейнер |
| `container_stop` | Остановить контейнер |
| `module_reload` | Горячая перезагрузка модуля |
| `agent_start_task` | Начать выполнение задачи |
| `security_validate` | Проверка политики безопасности |

### Keyboard Shortcuts

- `Ctrl+Shift+T`: Новый терминал
- `Ctrl+Shift+K`: Открыть ChatPanel
- `Ctrl+Shift+S`: Семантический поиск
- `F5`: Перезагрузка приложения

---

## Security Features

### Audit Logging

Все действия агента логируются с:
- Timestamp
- Хэш операции
- Rationale (обоснование)
- Результат выполнения

Логи доступны в `logs/audit.json`.

### Rate Limiting

LLM-запросы ограничены для предотвращения злоупотреблений:
- Максимум 10 запросов в минуту
- Блокировка при превышении на 5 минут

### Filesystem Whitelist

Доступ разрешен только к директориям:
- Проектная директория
- `~/iskin_modules/`
- Временные файлы

### Sandboxed Execution

Опасные операции выполняются в Docker:
- Компиляция модулей
- Выполнение shell-команд с риском
- Сетевые операции

---

## Troubleshooting

### LLM Not Starting

1. Проверьте путь к модели в `config/llm.toml`
2. Убедитесь, что llama.cpp установлен:
   ```bash
   which llama-server
   ```
3. Проверьте логи: `logs/llm.log`

### Container Issues

1. Убедитесь, что Docker запущен:
   ```bash
   docker ps
   ```
2. Проверьте права доступа к Docker socket
3. Перезапустите ContainerManager через UI

### Hot-Reload Failures

1. Проверьте компиляцию модуля:
   ```bash
   cargo build --release
   ```
2. Убедитесь, что `.so`/`.dll` создан
3. Проверьте `modules/versions.json` для rollback

### Performance Issues

1. Включите profiling:
   ```bash
   cargo run --features profiling
   ```
2. Проанализируйте flamegraph
3. Уменьшите `context_length` в конфиге

---

## Advanced Topics

### Custom Modules

Создание собственного модуля:

1. Создайте файл `my_module.rs`:
   ```rust
   use iskin_core::Module;
   
   #[no_mangle]
   pub extern "C" fn module_init() -> *mut Module {
       // Инициализация
   }
   ```

2. Скомпилируйте:
   ```bash
   cargo build --release
   ```

3. Загрузите через `module_reload`

### Vector Store Configuration

Для использования Qdrant:

```toml
[vector_store]
type = "qdrant"
url = "http://localhost:6333"
collection = "iskin_knowledge"

# Или локальный TF-IDF fallback
[vector_store]
type = "tfidf"
max_docs = 10000
```

### Model Profiles

Доступные профили моделей:

- `Qwen2.5-Coder-14B`: Кодогенерация
- `Llama-3-70B`: Общие задачи
- `Mistral-7B`: Быстрые ответы
- `Custom`: Свои настройки

---

## FAQ

**Q: Работает ли ISKIN без интернета?**
A: Да, все компоненты работают локально. LLM запускается на вашем ПК.

**Q: Можно ли использовать облачные LLM?**
A: Да, измените `host` и `port` в `config/llm.toml`.

**Q: Как откатить изменения агента?**
A: Используйте `modules/versions.json` для rollback к предыдущей версии.

**Q: Безопасно ли давать агенту доступ к файлам?**
A: Да, все операции проходят через PolicyEngine и AuditLogger.

**Q: Сколько памяти требуется для Qwen2.5-Coder-14B?**
A: Минимум 16GB RAM, рекомендуется 32GB.

---

## Support & Community

- **GitHub**: https://github.com/rexo-null/AI-Ecosystem-Interface
- **Issues**: Сообщайте о багах через GitHub Issues
- **Discussions**: Обсуждения и вопросы

## License

MIT License — см. LICENSE файл.
