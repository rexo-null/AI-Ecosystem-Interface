# AI Ecosystem Interface - Инструкция по запуску

## Требования

- Node.js 18+ 
- llama.cpp (для работы с нейросетью)
- Модель Qwen 2.5 14b Coder в формате GGUF

## Структура проекта

```
ai-ecosystem-interface/
├── client/          # Frontend на React + Monaco Editor
├── server/          # Backend на Express.js
└── README.md
```

## Установка

### 1. Установка зависимостей

```bash
# Frontend
cd client
npm install

# Backend
cd ../server
npm install
```

### 2. Настройка llama.cpp

#### Установка llama.cpp

```bash
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
make
```

#### Скачивание модели

Скачайте модель Qwen 2.5 14b Coder в формате GGUF:
- [HuggingFace](https://huggingface.co/Qwen)
- Или используйте `llama-cli` для конвертации

Разместите модель по пути: `./models/qwen-2.5-14b-coder.gguf`

### 3. Переменные окружения (опционально)

Для сервера можно настроить:

```bash
export WORKSPACE_ROOT="/path/to/your/project"
export LLAMA_CPP_PATH="/path/to/llama.cpp/main"
export MODEL_PATH="/path/to/qwen-2.5-14b-coder.gguf"
export PORT=5000
```

## Запуск

### Вариант 1: Раздельный запуск

```bash
# Терминал 1 - Backend
cd server
npm start

# Терминал 2 - Frontend
cd client
npm run dev
```

### Вариант 2: Одновременный запуск (требуется concurrently)

```bash
npm install -g concurrently
concurrently "cd server && npm start" "cd client && npm run dev"
```

## Доступ

После запуска:
- Frontend: http://localhost:3000
- Backend API: http://localhost:5000/api

## Функционал

### Редактор кода
- Подсветка синтаксиса для множества языков
- Вкладки для нескольких файлов
- Автосохранение

### Файловая система
- Просмотр дерева проекта
- Чтение/запись файлов
- Удаление файлов

### Чат с AI
- Интеграция с Qwen 2.5 14b Coder
- Контекстная помощь по коду
- Генерация кода по описанию

### Консоль
- Выполнение системных команд
- Вывод stdout/stderr

## Безопасность

⚠️ **Внимание**: Сервер имеет доступ к файловой системе и консоли. Не запускайте на публичных серверах без дополнительной аутентификации!

## Развитие

### Планы
- [ ] Добавить аутентификацию
- [ ] Поддержка нескольких моделей
- [ ] Streaming ответов от AI
- [ ] Расширенные настройки редактора
- [ ] Плагины и расширения
- [ ] Terminal in browser

## Лицензия

MIT
