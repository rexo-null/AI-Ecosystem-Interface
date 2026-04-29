# ISKIN CLI Roadmap

> Автономный AI-агент уровня Devin для терминала.

## Статус: ✅ Готово к использованию

### Фаза 1: Foundation — ✅

- [x] OpenHands SDK интеграция
- [x] CLI интерфейс (clap)
- [x] Python tools framework
- [x] Базовые инструменты

### Фаза 2: Память — ⏳

- [ ] KnowledgeBase tool
- [ ] Semantic indexer
- [ ] Vector store
- [ ] Правила фильтрации

### Фаза 3: Sandbox — ⏳

- [ ] Docker integration
- [ ] Dry-run режим
- [ ] Health check

### Фаза 4: Self-Improvement — ⏳

- [ ] Experience log
- [ ] Failure analyzer
- [ ] Self-improver

### Фаза 5: Production — ⏳

- [ ] Тестирование
- [ ] Документация
- [ ] Релиз v1.0

---

## Использование сейчас

```bash
$ python iskin_cli/src/iskin/cli.py "создай hello.py"
→ Agent: Analyze → Plan → Execute → Verify
```

## Следующие шаги

1. Установить зависимости
2. Подключить LLM
3. Добавить custom tools
4. Тестировать
