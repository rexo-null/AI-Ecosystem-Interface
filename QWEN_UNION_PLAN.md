# ✅ Синхронизация завершена

Теперь я полностью понимаю архитектуру и статус проекта. Спасибо, что направил.

---

## 🎯 Финальная цель (зафиксировано)

**Продукт:** `iskin` — автономный локальный AI-агент уровня Devin.  
**Интерфейс:** Сначала IDE (Tauri + React), затем опционально CLI/TUI как легковесная обёртка.  
**Ядро:** `src-tauri/` на Rust с горячим перезагрузкой модулей, песочницей, памятью и политикой безопасности.

---

## 📊 Текущий статус проекта (по ROADMAP.md)

| Фаза | Статус | Что реализовано |
|------|--------|-----------------|
| **1. Foundation** | ✅ DONE | Tauri v2 + React, LifecycleManager (hot-reload), PolicyEngine, ResourceManager, i18n |
| **2. Memory & Context** | ✅ DONE | KnowledgeBase, SemanticIndexer (Tree-sitter), VectorStore (Qdrant), RulesEngine |
| **3. Sandbox** | ✅ DONE | ContainerManager (Docker/bollard), VncManager (KasmVNC), BrowserAutomation (CDP), SelfHealingLoop |
| **4. LLM Integration** | ✅ DONE | HTTP-клиент к llama.cpp (SSE streaming), config/llm.toml, setup/run скрипты |
| **5. Interactive Terminal** | ✅ DONE | PTY Manager (portable-pty), xterm.js, мульти-табы, интеграция с ChatPanel |
| **6. Autonomous Agent** | 🔄 IN PROGRESS | Следующий приоритет: State Machine, Tool Protocol, Context Management |
| **7. Self-Improvement** | ⏳ Planned | Hot-reload модулей, компиляция в Docker, WASM sandbox |
| **8-10. Security/Polish/Butler** | ⏳ Future | RBAC, audit log, системный ассистент |

---

## 🧭 Где мы сейчас

**Текущий фокус: Фаза 6 — Autonomous Agent (Devin-level)**

Нужно реализовать:
1.  **Agent State Machine** (9 фаз): `ReceiveTask → Decompose → ImpactAssessment → DryRun → Execute → Verify → ArtifactSync → Commit → QueueNext`
2.  **Tool Use Protocol**: LLM → JSON Schema → PolicyEngine → Tool Call → Result → LLM
3.  **Context Management**: автосаммаризация, скользящее окно, критические факты
4.  **Agent UI**: статус-бар, панель задач, лог действий, ImpactReport view

---

## 🔗 Как это соотносится с `iskin-bridge`

`iskin-bridge`, который мы разрабатывали отдельно — это **прототип ядра агента**. Теперь его логика должна быть **интегрирована напрямую в `src-tauri/src/agent/`**:

```
AI-Ecosystem-Interface/
└── src-tauri/
    └── src/
        ├── agent/              # ← сюда переносим iskin-bridge логику
        │   ├── mod.rs
        │   ├── state_machine.rs    # AgentPhase enum, переходы
        │   ├── planner.rs          # Декомпозиция задач
        │   ├── validator.rs        # Валидация ответов/кода
        │   ├── context_mgr.rs      # Управление контекстом
        │   ├── tool_protocol.rs    # JSON Schema → PolicyEngine → Tools
        │   └── loop.rs             # Главный цикл агента
        ├── llm/                # Уже есть: HTTP-клиент к llama.cpp
        ├── tools/              # Уже есть: реестр инструментов
        ├── policy/             # Уже есть: PolicyEngine (RBAC)
        ├── sandbox/            # Уже есть: Docker, VNC, Browser
        └── api/commands.rs     # Tauri commands для фронтенда
```

**Преимущество:** Мы не начинаем с нуля. У нас уже есть:
- ✅ Готовый HTTP-клиент к llama.cpp (`llm.rs`)
- ✅ Реестр инструментов (`tools/`)
- ✅ PolicyEngine с уровнями риска
- ✅ Песочница (Docker) для безопасного выполнения
- ✅ Горячая перезагрузка модулей (LifecycleManager)

Осталось добавить **мозг агента** (планировщик, валидатор, цикл) и **связать всё воедино**.

---

## 🚀 План действий (синхронизированный)

### Шаг 1: Создать скелет `src/agent/`
- [ ] `mod.rs` с ре-экспортами
- [ ] `state_machine.rs` с `AgentPhase` enum и правилами переходов
- [ ] `loop.rs` с базовым циклом (пока заглушки)

### Шаг 2: Интегрировать Planner/Validator из iskin-bridge
- [ ] `planner.rs`: декомпозиция задач на атомарные шаги
- [ ] `validator.rs`: проверка полноты ответа, синтаксиса, выполнения
- [ ] `context_mgr.rs`: управление «чистым» контекстом на шаг

### Шаг 3: Tool Protocol + PolicyEngine
- [ ] JSON Schema валидация tool calls ДО исполнения
- [ ] Интеграция с существующим `PolicyEngine` (safe/confirm/dangerous)
- [ ] Обработка `tool_validation_error` без падения агента

### Шаг 4: Контекст и память
- [ ] `ContextCompressor`: автосаммаризация при >6K токенов
- [ ] Интеграция с `KnowledgeBase` для хранения длинных логов
- [ ] Скользящее окно истории (system prompt + последние N + резюме)

### Шаг 5: UI + Tauri Commands
- [ ] AgentStatusBar компонент (React)
- [ ] TaskPanel с прогрессом и подзадачами
- [ ] Action log streaming через Tauri events
- [ ] ImpactReport view перед выполнением

---

## ❓ Что делаем прямо сейчас?

Учитывая, что:
1.  Фаза 5 (Terminal) завершена
2.  Фаза 6 (Agent) — следующий приоритет
3.  У нас уже есть прототип ядра (`iskin-bridge`)

**Предлагаю начать с Шага 1: скелет `src/agent/`.**

Я могу подготовить:
- **Полный код `src/agent/mod.rs` + `state_machine.rs`** (AgentPhase enum, переходы, правила)
- **Интеграцию с существующими модулями** (llm, tools, policy)
- **Tauri commands для управления агентом** (`agent_start_task`, `agent_status`, `agent_interrupt`)

Или, если предпочтёшь, можем сначала **перенести Planner/Validator из iskin-bridge** в `src/agent/`.

**Что выбираешь?**
- [ ] **Вариант А:** Начинаем со скелета (`state_machine.rs` + `loop.rs`)
- [ ] **Вариант Б:** Сначала переносим Planner/Validator из iskin-bridge
- [ ] **Вариант В:** Другое (опиши)

Жду твоего решения — и я отдам полный код, готовый к `git add`. 🔧✨
