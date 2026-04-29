# 📘 ISKIN Bridge: План реализации системы управления контекстом

**Для:** Агент-разработчик (Continue / Copilot)  
**Версия:** 1.0  
**Приоритет:** Критический  

---

## 🎯 Цель и проблема

### ❌ Проблема (что ломается сейчас)

| Симптом | Причина | Следствие |
|---------|---------|-----------|
| Ответ модели обрывается на полуслове | Ограничение `--ctx-size` (8K–32K токенов) | Неполный код, синтаксические ошибки, потеря логики |
| Модель «забывает» начало задачи при длинном диалоге | Контекст забивается историей, рассуждениями, логами | Падение качества, зацикливание, галлюцинации |
| Невозможно сгенерировать файл >200 строк за один запрос | Линейная генерация в одном контекстном окне | Фрагментация кода, ручная сборка, ошибки интеграции |
| Ошибка в шаге ломает весь процесс | Нет изоляции ошибок, нет отката | Агент зависает, требует ручного вмешательства |

### ✅ Цель (что получим после реализации)

**ISKIN Bridge превращается из «линейного чата» в «иерархического автономного агента»**, который:

1.  **Разбивает любую задачу** на атомарные, валидируемые шаги (Планировщик)
2.  **Проверяет каждый ответ** перед сохранением: полнота, синтаксис, выполнение (Валидатор)
3.  **Очищает контекст** между шагами, загружая только релевантные артефакты (Менеджер контекста)
4.  **Изолирует ошибки**: сбой в шаге → создаётся подзадача «Исправить» → процесс продолжается
5.  **Генерирует код любого размера**: шаг за шагом, с чекпоинтами и агрегацией результата

**Итог:** Агент может работать над крупными файлами, рефакторингом и архитектурными задачами **без упора в лимиты контекста**, с автоматическим восстановлением после ошибок.

---

## 🗺️ Пошаговый план реализации

**Порядок строгой:** `Планировщик` → `Валидатор` → `Менеджер контекста` → `Интеграция`.

Каждый шаг — отдельный модуль, компилируемый и тестируемый независимо.

---

### 🔹 Шаг 1: Планировщик (`src/agent/planner.rs`)

**Задача:** Декомпозиция пользовательской задачи на атомарные, исполняемые шаги.

#### 📦 Типы данных

```rust
// Уникальный идентификатор шага
pub type TaskId = uuid::Uuid;

// Тип шага (атомарная операция)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStep {
    /// Генерация/правка кода в файле
    GenerateCode {
        file_path: String,
        description: String,
        start_line: Option<usize>, // Для инкрементальной правки
        end_line: Option<usize>,
    },
    /// Выполнение команды в терминале
    RunCommand {
        command: String,
        cwd: Option<String>,
        timeout_sec: u64,
        expected_exit_code: Option<i32>,
    },
    /// Валидация артефакта (код, конфиг, документ)
    Validate {
        artifact_path: String,
        rules: Vec<ValidationRule>,
    },
    /// Исправление ошибки (рекурсивная подзадача)
    FixError {
        error_log: String,
        original_step_id: TaskId,
        suggested_fix: Option<String>,
    },
    /// Агрегация результатов (финальный шаг)
    Aggregate {
        output_format: OutputFormat,
        include_steps: Vec<TaskId>,
    },
}

// Правила валидации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    SyntaxCheck(Language), // rust, toml, json, etc.
    CompilationCheck,      // cargo check / build
    TestPass,              // cargo test
    FileExists,
    RegexMatch(String),
    Custom(String), // имя кастомной проверки
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Rust, Toml, Json, Yaml, Markdown, Shell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Text, Json, Markdown, CodeBlock(String), // language hint
}
```

#### 🧠 Логика планировщика

```rust
pub struct Planner {
    config: PlannerConfig,
    llm_client: LLMClient, // Заглушка на Шаге 1, реальная на Шаге 3
}

pub struct PlannerConfig {
    pub max_subtasks: usize, // Лимит шагов на задачу (защита от бесконечной декомпозиции)
    pub prefer_atomic: bool, // Приоритет атомарным шагам (один файл / одна команда)
}

impl Planner {
    pub fn new(config: PlannerConfig) -> Self { /* ... */ }
    
    /// Декомпозирует задачу на шаги
    /// 
    /// Алгоритм:
    /// 1. Если задача простая (одно действие) → вернуть один шаг
    /// 2. Если сложная → запросить у модели план в структурированном формате
    /// 3. Распарсить ответ, валидировать структуру
    /// 4. Вернуть вектор шагов с зависимостями
    pub fn decompose(
        &self,
        task: &str,
        context: &AgentContext,
        artifacts: &ArtifactStore,
    ) -> Result<Vec<TaskStep>, PlannerError>;
    
    /// Создаёт подзадачу исправления ошибки
    pub fn create_fix_step(
        &self,
        error: &ExecutionError,
        original_step: &TaskStep,
    ) -> TaskStep;
}
```

#### 📝 Промпт для модели (внутри `decompose`)

```text
Ты — планировщик задач ISKIN. Твоя цель — разбить пользовательский запрос на атомарные, исполняемые шаги.

ПРАВИЛА:
1. Каждый шаг должен быть независимым: один файл, одна команда, одна проверка.
2. Избегай шагов типа "написать весь проект" — разбивай на файлы/модули.
3. Для каждого шага укажи: тип, параметры, ожидаемый результат.
4. Если задача требует итераций (например, "исправить ошибки компиляции") — создай шаг Validate + цикл Fix.

ФОРМАТ ОТВЕТА (строгий JSON):
{
  "steps": [
    {
      "type": "GenerateCode",
      "params": {
        "file_path": "src/agent/planner.rs",
        "description": "Реализовать структуру TaskStep с вариантами...",
        "start_line": null,
        "end_line": null
      }
    },
    {
      "type": "Validate",
      "params": {
        "artifact_path": "src/agent/planner.rs",
        "rules": ["SyntaxCheck(Rust)", "CompilationCheck"]
      }
    }
  ],
  "estimated_steps": 2
}

Не добавляй пояснений вне JSON.
```

#### ✅ Критерии готовности Шага 1

- [ ] `planner.rs` компилируется без ошибок
- [ ] `Planner::decompose()` возвращает валидный вектор `TaskStep`
- [ ] Юнит-тесты на парсинг простых и сложных задач
- [ ] Интеграционный тест: задача "создать файл с кодом" → план из 2 шагов (генерация + валидация)

---

### 🔹 Шаг 2: Валидатор (`src/agent/validator.rs`)

**Задача:** Проверка ответа модели **перед** сохранением результата и очисткой контекста.

#### 📦 Типы данных

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Ответ полный, код валиден, команда выполнилась
    Ok,
    
    /// Ответ обрывается (нет закрывающих скобок, неполный блок кода)
    Incomplete {
        partial_content: String, // То, что удалось распарсить
        continuation_hint: String, // Подсказка для модели: "продолжи с строки X"
    },
    
    /// Ошибка валидации
    Error {
        message: String,
        recoverable: bool, // Можно ли исправить автоматически
        error_type: ValidationErrorType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    SyntaxError { language: Language, line: Option<usize> },
    CompilationError { output: String },
    TestFailed { output: String },
    Timeout,
    PermissionDenied,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub step_id: TaskId,
    pub message: String,
    pub error_type: ValidationErrorType,
    pub recoverable: bool,
    pub context_snapshot: String, // Минимальный контекст для отладки
}
```

#### 🧠 Логика валидатора

```rust
pub struct Validator {
    workspace_root: PathBuf,
    allowed_languages: Vec<Language>,
}

impl Validator {
    pub fn new(workspace_root: PathBuf) -> Self { /* ... */ }
    
    /// Проверка полноты ответа (детект обрывов)
    pub fn check_completeness(response: &str, step: &TaskStep) -> ValidationResult {
        match step {
            TaskStep::GenerateCode { .. } => {
                // Проверка: есть ли открывающие/закрывающие скобки, блоки кода
                Self::check_code_balance(response)
            }
            TaskStep::RunCommand { .. } => {
                // Проверка: есть ли вывод команды, код возврата
                Self::check_command_output(response)
            }
            _ => ValidationResult::Ok,
        }
    }
    
    /// Проверка синтаксиса кода
    pub fn check_syntax(code: &str, language: Language) -> ValidationResult {
        match language {
            Language::Rust => Self::check_rust_syntax(code),
            Language::Json => Self::check_json_syntax(code),
            // ... другие языки
            _ => ValidationResult::Ok, // Неизвестный язык — пропускаем
        }
    }
    
    /// Проверка компиляции (для Rust)
    pub async fn check_compilation(file_path: &str, cwd: Option<&str>) -> ValidationResult {
        // Запуск: cargo check --manifest-path {cwd}/Cargo.toml
        // Парсинг вывода, детект ошибок
    }
    
    /// Главная точка входа
    pub async fn validate(
        &self,
        step: &TaskStep,
        response: &str,
        artifacts: &ArtifactStore,
    ) -> ValidationResult;
}
```

#### 🔍 Детект обрывов кода (пример для Rust)

```rust
fn check_code_balance(code: &str) -> ValidationResult {
    let mut braces = 0;
    let mut brackets = 0;
    let mut parens = 0;
    let mut in_string = false;
    let mut escape = false;
    
    for ch in code.chars() {
        if escape { escape = false; continue; }
        if ch == '\\' { escape = true; continue; }
        if ch == '"' || ch == '\'' { in_string = !in_string; continue; }
        if in_string { continue; }
        
        match ch {
            '{' => braces += 1,
            '}' => braces -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }
    }
    
    if braces < 0 || brackets < 0 || parens < 0 {
        return ValidationResult::Error {
            message: "Unbalanced closing bracket detected".into(),
            recoverable: true,
            error_type: ValidationErrorType::SyntaxError { language: Language::Rust, line: None },
        };
    }
    
    if braces > 0 || brackets > 0 || parens > 0 {
        return ValidationResult::Incomplete {
            partial_content: code.to_string(),
            continuation_hint: format!(
                "Code block is incomplete: {} unclosed braces, {} brackets, {} parens. Continue from last line.",
                braces, brackets, parens
            ),
        };
    }
    
    ValidationResult::Ok
}
```

#### ✅ Критерии готовности Шага 2

- [ ] `validator.rs` компилируется без ошибок
- [ ] `Validator::check_completeness()` детектит обрывы кода
- [ ] `Validator::check_syntax()` валидирует Rust/JSON
- [ ] Асинхронная `check_compilation()` запускает `cargo check` и парсит вывод
- [ ] Юнит-тесты на кейсы: полный код, обрыв, синтаксическая ошибка

---

### 🔹 Шаг 3: Менеджер контекста (`src/agent/context_mgr.rs`)

**Задача:** Управление «чистым» контекстом для каждого шага — загрузка только релевантных артефактов, сброс после валидации.

#### 📦 Типы данных

```rust
#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub base_prompt: String, // Системный промпт (не меняется)
    pub max_tokens_per_step: usize, // Лимит токенов на шаг (защита от переполнения)
    pub include_file_content: bool, // Включать ли полный контент файлов или только хеши
    pub max_history_messages: usize, // Сколько сообщений истории сохранять между шагами
}

#[derive(Debug, Clone)]
pub struct ArtifactReference {
    pub path: String,
    pub hash: String, // SHA-256 для детекта изменений
    pub relevance_score: f32, // 0.0..1.0 — насколько артефакт релевантен текущему шагу
}

pub struct ArtifactStore {
    artifacts: HashMap<String, ArtifactData>,
}

#[derive(Debug, Clone)]
pub struct ArtifactData {
    pub content: String,
    pub hash: String,
    pub last_modified: DateTime<Utc>,
    pub metadata: ArtifactMetadata,
}
```

#### 🧠 Логика менеджера контекста

```rust
pub struct ContextManager {
    config: ContextConfig,
    base_prompt: String,
    current_context: String, // Буфер текущего контекста (очищается между шагами)
}

impl ContextManager {
    pub fn new(config: ContextConfig) -> Self { /* ... */ }
    
    /// Строит контекст для конкретного шага
    /// 
    /// Алгоритм:
    /// 1. Начинаем с base_prompt
    /// 2. Добавляем описание текущего шага
    /// 3. Добавляем релевантные артефакты (файлы, результаты предыдущих шагов)
    /// 4. Обрезаем по лимиту max_tokens_per_step (с приоритетом на новые данные)
    /// 5. Возвращаем готовый промпт для модели
    pub fn build_for_step(
        &mut self,
        step: &TaskStep,
        artifacts: &ArtifactStore,
        previous_results: &[StepResult],
    ) -> Result<String, ContextError>;
    
    /// Очищает контекст (сбрасывает буфер, оставляя только base_prompt)
    pub fn clear(&mut self);
    
    /// Подсчитывает токены в строке (приблизительно)
    pub fn count_tokens(text: &str) -> usize;
    
    /// Обрезает текст до лимита токенов, сохраняя целостность строк/блоков
    pub fn truncate_to_tokens(text: &str, max_tokens: usize) -> String;
}
```

#### 🔁 Цикл очистки контекста

```rust
// Псевдокод интеграции в AgentLoop
async fn execute_step(&mut self, step: TaskStep) -> Result<StepResult> {
    // 1. Подготовка "чистого" контекста
    let prompt = self.ctx_mgr.build_for_step(&step, &self.artifacts, &self.results)?;
    
    // 2. Запрос к модели
    let response = self.llm.generate(&prompt).await?;
    
    // 3. Валидация
    match self.validator.validate(&step, &response, &self.artifacts).await {
        ValidationResult::Ok => {
            // 4. Сохранение результата
            self.artifacts.store(step.id(), &response);
            
            // 5. ОЧИСТКА КОНТЕКСТА ← ключевой момент
            self.ctx_mgr.clear();
            
            Ok(StepResult::Success(response))
        }
        ValidationResult::Incomplete { partial, hint } => {
            // Частичный успех: сохраняем то, что есть, предлагаем продолжить
            self.artifacts.store_partial(step.id(), &partial);
            self.ctx_mgr.clear(); // Очищаем, чтобы не тащить "мусор" в следующий запрос
            Ok(StepResult::Partial { partial, continuation_hint: hint })
        }
        ValidationResult::Error { message, recoverable, .. } => {
            // Ошибка: НЕ очищаем контекст полностью — сохраняем снапшот для отладки
            let snapshot = self.ctx_mgr.snapshot_for_debug();
            self.artifacts.store_error(step.id(), &message, &snapshot);
            
            if recoverable {
                // Создаём подзадачу исправления
                let fix_step = self.planner.create_fix_step(&message, &step);
                self.queue.push_front(fix_step); // Приоритетная вставка
            }
            
            Ok(StepResult::Failed { error: message })
        }
    }
}
```

#### ✅ Критерии готовности Шага 3

- [ ] `context_mgr.rs` компилируется без ошибок
- [ ] `ContextManager::build_for_step()` собирает промпт с релевантными артефактами
- [ ] `ContextManager::clear()` сбрасывает буфер, оставляя base_prompt
- [ ] Реализован подсчёт токенов (приблизительный, по словам/символам)
- [ ] Интеграционный тест: два шага подряд → контекст не "переполняется"

---

### 🔹 Шаг 4: Интеграция и чекпоинты (`src/agent/checkpoint.rs`)

**Задача:** Сохранение состояния между шагами, чтобы очистка контекста не теряла прогресс.

#### 📦 Минимальная реализация

```rust
// checkpoint.rs
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub task_id: uuid::Uuid,
    pub step_id: TaskId,
    pub step_type: String, // Для быстрого фильтра
    pub input: String,
    pub output: Option<String>,
    pub validation_result: Option<ValidationResult>,
    pub artifacts_created: Vec<String>, // Пути к новым/изменённым файлам
    pub timestamp: DateTime<Utc>,
    pub error: Option<ExecutionError>,
}

pub struct CheckpointManager {
    storage_path: PathBuf,
}

impl CheckpointManager {
    pub fn new(storage_path: PathBuf) -> Self { /* ... */ }
    
    /// Сохраняет чекпоинт перед очисткой контекста
    pub fn save(&self, checkpoint: Checkpoint) -> Result<()>;
    
    /// Загружает историю чекпоинтов для задачи
    pub fn load_task_history(&self, task_id: uuid::Uuid) -> Result<Vec<Checkpoint>>;
    
    /// Восстанавливает состояние по чекпоинту (для отката)
    pub fn restore(&self, checkpoint_id: uuid::Uuid) -> Result<RestoreResult>;
}
```

#### 🔗 Интеграция в цикл

```rust
// В AgentLoop::execute_step(), после ValidationResult::Ok:
let checkpoint = Checkpoint {
    task_id: self.task_id,
    step_id: step.id(),
    step_type: format!("{:?}", step),
    input: prompt.clone(),
    output: Some(response.clone()),
    validation_result: Some(validation),
    artifacts_created: new_files.clone(),
    timestamp: Utc::now(),
    error: None,
};
self.checkpoint_mgr.save(checkpoint)?;

// Теперь можно безопасно очищать контекст
self.ctx_mgr.clear();
```

#### ✅ Критерии готовности Шага 4

- [ ] `checkpoint.rs` компилируется
- [ ] Чекпоинты сохраняются в JSON-файлы в `logs/checkpoints/`
- [ ] Интеграция: после каждого успешного шага создаётся чекпоинт
- [ ] Тест: перезапуск агента → загрузка истории → продолжение с места сбоя

---

## 🔄 Полный поток выполнения задачи (после всех шагов)

```
[Пользователь: "Создай модуль валидации для ISKIN"]
        │
        ▼
[Planner::decompose()]
├─ Шаг 1: GenerateCode { file: "src/agent/validator.rs", ... }
├─ Шаг 2: Validate { artifact: "src/agent/validator.rs", rules: [Syntax, Compilation] }
├─ Шаг 3: GenerateCode { file: "src/agent/validator.rs", description: "Добавить тесты", ... }
└─ Шаг 4: Validate { artifact: "src/agent/validator.rs", rules: [TestPass] }
        │
        ▼
[Цикл выполнения каждого шага]
┌─────────────────────────────────┐
│ Шаг 1:                          │
│  • ContextMgr: чистый контекст  │
│  • LLM: генерирует код          │
│  • Validator: синтаксис ОК      │
│  • Checkpoint: сохранено        │
│  • ContextMgr: clear()          │
│  → Результат: файл создан       │
│                                 │
│ Шаг 2:                          │
│  • ContextMgr: + файл из Шага 1 │
│  • LLM: проверяет код           │
│  • Validator: cargo check ОК    │
│  • Checkpoint: сохранено        │
│  • ContextMgr: clear()          │
│  → Результат: код валиден       │
│                                 │
│ Шаг 3: ... (аналогично)         │
└─────────────────────────────────┘
        │
        ▼
[Агрегация] → "Модуль валидации создан, тесты пройдены"
```

---

## 📋 Чек-лист готовности всей системы

- [ ] **Планировщик**: декомпозиция задач, создание подзадач исправления
- [ ] **Валидатор**: детект обрывов, проверка синтаксиса/компиляции/тестов
- [ ] **Менеджер контекста**: сбор релевантного контекста, очистка между шагами
- [ ] **Чекпоинты**: сохранение состояния, возможность отката
- [ ] **Интеграция в AgentLoop**: цикл `план → выполнить → валидировать → очистить → следующий`
- [ ] **Обработка ошибок**: изоляция, создание подзадач, логирование
- [ ] **Тесты**: юнит-тесты на каждый модуль + интеграционный тест на задачу "создать файл"

---

## 🚀 Инструкция для агента (как работать с этим планом)

1.  **Начни с Шага 1** (`planner.rs`). Не переходи к следующему, пока текущий не скомпилируется и не пройдёт тесты.
2.  **Для каждого файла**: 
    - Создай файл по пути `src/agent/{имя}.rs`
    - Вставь полный код (без фрагментов)
    - Добавь модуль в `src/agent/mod.rs`: `pub mod {имя};`
    - Запусти `cargo check` — исправь ошибки, если есть
3.  **После каждого шага**: напиши «Шаг {N} готов», и я подтвержу или дам правки.
4.  **Если модель обрывает код**: используй валидатор из Шага 2, чтобы детектить обрыв и запросить продолжение.
5.  **При интеграции**: обновляй `AgentLoop` постепенно, не меняй всё сразу.

---

## 💡 Примечания для разработчика

- **Атомарность — ключ**: каждый шаг должен делать одно маленькое дело. Лучше 10 шагов по 10 строк кода, чем 1 шаг на 100 строк с риском обрыва.
- **Валидация до очистки**: никогда не очищай контекст, пока не убедился, что результат сохранён и валиден.
- **Логируй всё**: каждый шаг, каждая валидация, каждая очистка — в `audit.log`. Это спасёт при отладке.
- **Тестируй на реальных кейсах**: "создать файл", "исправить ошибку компиляции", "добавить функцию в существующий код".

---

*Этот план — дорожная карта от «чата с моделью» к «автономному инженеру». Следуй шагам последовательно, и ты получишь систему, которая не упирается в лимиты контекста.* 🔧✨