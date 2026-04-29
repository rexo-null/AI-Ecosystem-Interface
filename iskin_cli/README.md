# ISKIN CLI

Autonomous AI Agent on OpenHands SDK with local LLM.

## Installation

```bash
cd iskin_cli
pip install -e .
```

## Usage

```bash
# Single prompt
iskin "создай файл test.txt"

# Interactive mode
iskin --interactive

# Dry run (simulate)
iskin --dry-run "удалить все .tmp"
```

## Configuration

Edit `config/llm.toml`:

```toml
[llm]
endpoint = "http://localhost:8080"
model = "qwen2.5-coder"
```
