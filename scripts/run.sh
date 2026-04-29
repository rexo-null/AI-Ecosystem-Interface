#!/usr/bin/env bash
# ============================================================
# ISKIN Run Script (Linux / macOS)
# Запускает llama-server и ISKIN
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$PROJECT_DIR/config/llm.toml"

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ============================================================
# Чтение конфигурации из config/llm.toml
# ============================================================
# Значения по умолчанию
MODEL_PATH="models/qwen2.5-coder-14b-instruct-Q4_K_M.gguf"
HOST="127.0.0.1"
PORT="8080"
CTX_SIZE="8192"
GPU_LAYERS="45"
THREADS="8"
BATCH_SIZE="512"
UBATCH_SIZE="64"
FLASH_ATTN="true"
CACHE_TYPE_K="q4_0"

if [ -f "$CONFIG_FILE" ]; then
    info "Чтение конфигурации из $CONFIG_FILE"

    # Простой парсинг TOML (значения без кавычек и с кавычками)
    parse_toml() {
        local key="$1"
        local val
        val=$(grep -E "^${key}\s*=" "$CONFIG_FILE" | head -1 | sed 's/.*=\s*//' | sed 's/^"//' | sed 's/"$//' | sed 's/\s*#.*//' | tr -d '[:space:]')
        echo "$val"
    }

    _val=$(parse_toml "endpoint"); [ -n "$_val" ] && {
        HOST=$(echo "$_val" | sed 's|http://||' | cut -d: -f1)
        PORT=$(echo "$_val" | sed 's|http://||' | cut -d: -f2)
    }
    _val=$(parse_toml "path"); [ -n "$_val" ] && MODEL_PATH="$_val"
    _val=$(parse_toml "context_length"); [ -n "$_val" ] && CTX_SIZE="$_val"
    _val=$(parse_toml "gpu_layers"); [ -n "$_val" ] && GPU_LAYERS="$_val"
    _val=$(parse_toml "threads"); [ -n "$_val" ] && THREADS="$_val"
    _val=$(parse_toml "batch_size"); [ -n "$_val" ] && BATCH_SIZE="$_val"
    _val=$(parse_toml "ubatch_size"); [ -n "$_val" ] && UBATCH_SIZE="$_val"
    _val=$(parse_toml "flash_attn"); [ -n "$_val" ] && FLASH_ATTN="$_val"
    _val=$(parse_toml "cache_type_k"); [ -n "$_val" ] && CACHE_TYPE_K="$_val"
else
    warn "Конфигурация не найдена: $CONFIG_FILE (используются значения по умолчанию)"
fi

# Полный путь к модели
if [[ ! "$MODEL_PATH" = /* ]]; then
    MODEL_PATH="$PROJECT_DIR/$MODEL_PATH"
fi

# ============================================================
# Поиск llama-server
# ============================================================
LLAMA_SERVER=""

if command -v llama-server &>/dev/null; then
    LLAMA_SERVER="llama-server"
elif [ -f "$PROJECT_DIR/llama.cpp/build/bin/llama-server" ]; then
    LLAMA_SERVER="$PROJECT_DIR/llama.cpp/build/bin/llama-server"
else
    error "llama-server не найден!"
    echo "Запустите ./scripts/setup.sh для установки."
    exit 1
fi

ok "llama-server: $LLAMA_SERVER"

# ============================================================
# Проверка модели
# ============================================================
if [ ! -f "$MODEL_PATH" ]; then
    error "Модель не найдена: $MODEL_PATH"
    echo "Запустите ./scripts/setup.sh для скачивания модели."
    exit 1
fi

ok "Модель: $MODEL_PATH ($(du -h "$MODEL_PATH" | cut -f1))"

# ============================================================
# Запуск llama-server
# ============================================================
info "Запуск llama-server..."
echo "  Host: $HOST:$PORT"
echo "  Model: $(basename "$MODEL_PATH")"
echo "  Context: $CTX_SIZE tokens"
echo "  GPU Layers: $GPU_LAYERS"
echo "  Threads: $THREADS"

# Сборка аргументов
LLAMA_ARGS=(
    --model "$MODEL_PATH"
    --ctx-size "$CTX_SIZE"
    --n-gpu-layers "$GPU_LAYERS"
    --threads "$THREADS"
    --batch-size "$BATCH_SIZE"
    --ubatch-size "$UBATCH_SIZE"
    --host "$HOST"
    --port "$PORT"
    --cache-type-k "$CACHE_TYPE_K"
)

if [ "$FLASH_ATTN" = "true" ]; then
    LLAMA_ARGS+=(--flash-attn)
fi

# Запуск в фоне
"$LLAMA_SERVER" "${LLAMA_ARGS[@]}" &
LLAMA_PID=$!

# Ожидание готовности
info "Ожидание готовности llama-server..."
MAX_WAIT=60
WAITED=0
while [ $WAITED -lt $MAX_WAIT ]; do
    if curl -s "http://$HOST:$PORT/v1/models" >/dev/null 2>&1; then
        ok "llama-server готов (PID: $LLAMA_PID)"
        break
    fi
    sleep 2
    WAITED=$((WAITED + 2))
    echo -n "."
done

if [ $WAITED -ge $MAX_WAIT ]; then
    error "llama-server не запустился за ${MAX_WAIT}с"
    kill $LLAMA_PID 2>/dev/null || true
    exit 1
fi

echo ""

# ============================================================
# Запуск ISKIN (Tauri dev)
# ============================================================
info "Запуск ISKIN..."
cd "$PROJECT_DIR"

# Обработка завершения: остановить llama-server при выходе
cleanup() {
    info "Остановка llama-server (PID: $LLAMA_PID)..."
    kill $LLAMA_PID 2>/dev/null || true
    wait $LLAMA_PID 2>/dev/null || true
    ok "llama-server остановлен"
}
trap cleanup EXIT INT TERM

# Запуск Tauri в режиме разработки
npx tauri dev
