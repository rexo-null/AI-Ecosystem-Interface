# ISKIN CLI Roadmap

> Конечная цель: автономный self-improving AI-агент уровня Devin, работающий в терминале.

## Phase 1: Ядро (Weeks 1-3)

- [x] Agent State Machine (9 фаз)
- [x] LLM Client (HTTP → llama.cpp) с SSE streaming
- [x] Tool Registry и Executor
- [x] CLI каркас (clap)
- [x] Сессии (SQLite)
- [x] Context Compressor

## Phase 2: Песочница (Weeks 4-6)

- [x] Docker Manager (bollard)
- [x] Container exec / logs
- [x] Dry-run режим
- [x] Self-Healing Loop

## Phase 3: Память (Weeks 7-9)

- [x] Knowledge Base
- [x] Semantic Indexer
- [x] Vector Store
- [x] Rules Engine

## Phase 4: Secure Shell (Weeks 10-12)

- [x] PTY Manager
- [x] Shell commands
- [x] PolicyEngine (Safe/Confirm/Dangerous)

## Phase 5: Self-Improvement (Weeks 13-16)

- [x] Experience Log
- [x] Failure Analyzer
- [x] SelfImprover → обновление промптов
- [x] Module Hot-Reload
- [x] Dynamic tool loading

## Phase 6: CLI Polish (Weeks 17-20)

- [x] Completions (bash/zsh/fish)
- [x] Colored output
- [x] Configuration file
- [x] Plugins system
- [x] CI/CD интеграция

## Phase 7: Release

- [x] v1.0.0
- [x] Cross-platform binaries (Linux/macOS/Windows)
- [x] Documentation

---

## Статус: Готово к использованию

```
$ iskin ask "напиши функцию hello world на Rust"
→ Agent: ReceiveTask → Decompose → Execute → Verify → Done
```
