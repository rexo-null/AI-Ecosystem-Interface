# Инструкция по подключению LLM (Qwen-2.5-Coder-14B)

## Обновлено

✅ **Frontend (React)**
- Интерфейс полностью переведен на русский язык
- Система локализации (i18n) создана и работает
- Подключение к реальному LLM API через Tauri команду `chat_with_llm`
- ChatPanel теперь отправляет запросы на Rust backend

✅ **Backend (Rust/Tauri)**
- Создан модуль `llm.rs` с LLMEngine
- Добавлена команда `chat_with_llm` в API
- Инициализация LLM при старте приложения
- Система хранения истории чата

## Что нужно сделать для полной работы LLM

### 1. Установить зависимости LLM (в `src-tauri/Cargo.toml` уже добавлены)

Текущие зависимости для работы с моделями:
```toml
candle-core = "0.6"
candle-transformers = "0.6"
tokenizers = "0.19"
```

Для использования CUDA/GPU ускорения (опционально):
```toml
candle-core = { version = "0.6", features = ["cuda"] }
```

### 2. Скачать модель Qwen-2.5-Coder-14B

**Вариант A: Использование Hugging Face Hub**

```bash
# Установить huggingface-hub CLI
pip install huggingface-hub

# Скачать модель (требует 28GB памяти)
huggingface-cli download Qwen/Qwen2.5-Coder-14B-Instruct \
  --local-dir ./models/qwen-2.5-coder-14b
```

**Вариант B: Использование программиста для загрузки**

Создать Rust скрипт в `src-tauri/src/bin/download_model.rs`:

```rust
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let model_dir = Path::new("./models/qwen-2.5-coder-14b");
    fs::create_dir_all(model_dir)?;
    
    // Загрузить модель из Hugging Face
    // Примерная реализация будет добавлена
    
    Ok(())
}
```

### 3. Обновить `llm.rs` для использования реальной модели

Текущая реализация использует плейсхолдер. Нужно заменить `generate_response` для:

```rust
// Псевдокод для использования Candle
use candle_core::{Device, Tensor};
use candle_transformers::models::qwen2;

pub async fn generate_response(&self, request: &ChatRequest) -> anyhow::Result<LLMResponse> {
    let device = Device::cuda_if_available(0)?;
    
    // Загрузить модель
    let model_path = "models/qwen-2.5-coder-14b/model.safetensors";
    let model = QwenModel::new(&model_path, &device)?;
    
    // Подготовить промпт
    let prompt = format_Messages(&request.messages, request.system_prompt);
    
    // Генерировать ответ
    let response = model.generate(
        &prompt,
        request.temperature.unwrap_or(0.7),
        request.max_tokens.unwrap_or(2048),
    ).await?;
    
    Ok(LLMResponse {
        content: response,
        model: self.model_name.clone(),
        tokens_used: 0,
    })
}
```

### 4. Конфигурация для разработки

Создать файл `src-tauri/llm_config.json`:

```json
{
  "model": "Qwen-2.5-Coder-14B-Instruct",
  "model_path": "./models/qwen-2.5-coder-14b",
  "tokenizer_path": "./models/qwen-2.5-coder-14b/tokenizer.json",
  "device": "cuda",
  "dtype": "bfloat16",
  "max_seq_length": 8192,
  "temperature": 0.7,
  "top_p": 0.95,
  "max_tokens": 2048
}
```

### 5. Тестирование API

После компиляции Rust кода:

```bash
# Перестроить проект
cd src-tauri
cargo build --release

# Запустить фронтенд + бэкенд
cd ..
npm run tauri dev
```

Затем отправить сообщение в чат и проверить ответ от LLM.

## Текущий статус

| Компонент | Статус | Примечание |
|-----------|--------|-----------|
| Frontend (React) | ✅ | На русском, подключено к API |
| Backend (Tauri) | ✅ | LLM модуль создан, API команда готова |
| LLM модель | ⚠️ | Требует загрузки и интеграции |
| Chat интеграция | ✅ | Полностью функциональна |
| Файловая система | ⚠️ | Базовая поддержка |
| Система памяти | ✅ | Qdrant готов к использованию |

## Альтернативы

Если Candle слишком сложен:

### 1. llama.cpp (простее)
```bash
# Установить llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# Скачать GGUF модель (легче, 7-14GB)
# Хранить в models/

# Из Rust вызывать через subprocess:
use std::process::Command;

let output = Command::new("./llama-cli")
    .arg("-m").arg("models/model.gguf")
    .arg("-p").arg(&prompt)
    .output()?;
```

### 2. Использовать внешний API (во время разработки)
```rust
// Временный вариант для тестирования
use reqwest;

let response = reqwest::Client::new()
    .post("https://api.openrouter.io/api/v1/chat/completions")
    .header("Authorization", "Bearer YOUR_KEY")
    .json(&serde_json::json!({
        "model": "qwen/qwen-2.5-coder-14b",
        "messages": &request.messages,
    }))
    .send()
    .await?;
```

## Дальнейшее развитие

1. **Кэширование модели** - сохранение загруженной модели в памяти
2. **Потоковые ответы** - вывод ответа токен за токеном
3. **Параллельные запросы** - очередь обработки нескольких запросов
4. **Fine-tuning** - переобучение на специфичных для проекта данных
5. **Multimodal** - добавить Qwen-VL для анализа изображений

## Ресурсы

- [Qwen Documentation](https://qwen.readthedocs.io/)
- [Candle Examples](https://github.com/huggingface/candle/tree/main/candle-examples)
- [llama.cpp](https://github.com/ggerganov/llama.cpp)
- [Tauri Invoke](https://tauri.app/develop/calling-rust/)
