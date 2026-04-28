#!/usr/bin/env bash
# ============================================================
# ISKIN Setup Script (Linux / macOS)
# Устанавливает все зависимости для запуска ISKIN с llama.cpp
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MODELS_DIR="$PROJECT_DIR/models"
LLAMA_DIR="$PROJECT_DIR/llama.cpp"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ============================================================
# 1. Проверка системных зависимостей
# ============================================================
info "Проверка системных зависимостей..."

# Rust
if command -v rustc &>/dev/null; then
    RUST_VER=$(rustc --version)
    ok "Rust установлен: $RUST_VER"
else
    warn "Rust не найден. Устанавливаю..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    ok "Rust установлен: $(rustc --version)"
fi

# Node.js
if command -v node &>/dev/null; then
    NODE_VER=$(node --version)
    ok "Node.js установлен: $NODE_VER"
else
    error "Node.js не найден. Установите Node.js 18+ (https://nodejs.org/)"
    exit 1
fi

# npm
if command -v npm &>/dev/null; then
    ok "npm установлен: $(npm --version)"
else
    error "npm не найден."
    exit 1
fi

# Git
if command -v git &>/dev/null; then
    ok "Git установлен: $(git --version)"
else
    error "Git не найден."
    exit 1
fi

# CMake (для сборки llama.cpp)
if command -v cmake &>/dev/null; then
    ok "CMake установлен: $(cmake --version | head -1)"
else
    warn "CMake не найден. Устанавливаю..."
    if [[ "$(uname)" == "Linux" ]]; then
        sudo apt-get update && sudo apt-get install -y cmake
    elif [[ "$(uname)" == "Darwin" ]]; then
        brew install cmake
    fi
    ok "CMake установлен"
fi

# ============================================================
# 2. Сборка llama.cpp
# ============================================================
info "Подготовка llama.cpp..."

if [ -f "$LLAMA_DIR/build/bin/llama-server" ] || [ -f "$LLAMA_DIR/build/bin/llama-server.exe" ]; then
    ok "llama.cpp уже собран"
else
    if [ ! -d "$LLAMA_DIR" ]; then
        info "Клонирование llama.cpp..."
        git clone https://github.com/ggerganov/llama.cpp.git "$LLAMA_DIR"
    fi

    cd "$LLAMA_DIR"
    mkdir -p build && cd build

    # Определение GPU
    GPU_FLAGS=""
    if command -v rocminfo &>/dev/null; then
        info "Обнаружен ROCm (AMD GPU)"
        GPU_FLAGS="-DGGML_HIP=ON"
        ok "Сборка с поддержкой AMD GPU (ROCm)"
    elif command -v nvidia-smi &>/dev/null; then
        info "Обнаружен NVIDIA GPU"
        GPU_FLAGS="-DGGML_CUDA=ON"
        ok "Сборка с поддержкой NVIDIA GPU (CUDA)"
    elif [[ "$(uname)" == "Darwin" ]]; then
        info "macOS — используем Metal"
        GPU_FLAGS="-DGGML_METAL=ON"
        ok "Сборка с поддержкой Apple Metal"
    else
        warn "GPU не обнаружен — сборка только для CPU"
    fi

    info "Сборка llama.cpp (это может занять несколько минут)..."
    cmake .. $GPU_FLAGS -DLLAMA_BUILD_SERVER=ON
    cmake --build . --config Release -j "$(nproc 2>/dev/null || sysctl -n hw.ncpu)"

    ok "llama.cpp собран успешно"
    cd "$PROJECT_DIR"
fi

# ============================================================
# 3. Скачивание модели
# ============================================================
MODEL_FILE="$MODELS_DIR/qwen2.5-coder-14b-instruct-Q4_K_M.gguf"

info "Проверка модели Qwen2.5-Coder-14B..."

if [ -f "$MODEL_FILE" ]; then
    ok "Модель уже скачана: $(du -h "$MODEL_FILE" | cut -f1)"
else
    mkdir -p "$MODELS_DIR"
    info "Скачивание Qwen2.5-Coder-14B (Q4_K_M, ~9.2 GB)..."
    info "Это может занять значительное время в зависимости от скорости интернета."

    MODEL_URL="https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF/resolve/main/qwen2.5-coder-14b-instruct-q4_k_m.gguf"

    if command -v wget &>/dev/null; then
        wget -c -O "$MODEL_FILE" "$MODEL_URL"
    elif command -v curl &>/dev/null; then
        curl -L -C - -o "$MODEL_FILE" "$MODEL_URL"
    else
        error "Ни wget, ни curl не найдены. Скачайте модель вручную:"
        echo "  URL: $MODEL_URL"
        echo "  Путь: $MODEL_FILE"
        exit 1
    fi

    ok "Модель скачана: $(du -h "$MODEL_FILE" | cut -f1)"
fi

# ============================================================
# 4. Установка npm зависимостей
# ============================================================
info "Установка npm зависимостей..."
cd "$PROJECT_DIR"
npm install
ok "npm зависимости установлены"

# ============================================================
# 5. Проверка Tauri CLI
# ============================================================
if npx tauri --version &>/dev/null; then
    ok "Tauri CLI: $(npx tauri --version)"
else
    warn "Tauri CLI не найден в devDependencies. Проверьте package.json."
fi

# ============================================================
# Готово
# ============================================================
echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}  ISKIN Setup завершён успешно!${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo "Для запуска ISKIN выполните:"
echo "  ./scripts/run.sh"
echo ""
echo "Или запустите компоненты вручную:"
echo "  1. llama-server:  ./llama.cpp/build/bin/llama-server \\"
echo "       --model models/qwen2.5-coder-14b-instruct-Q4_K_M.gguf \\"
echo "       --ctx-size 8192 --n-gpu-layers 45 --host 127.0.0.1 --port 8080"
echo "  2. ISKIN:         cargo tauri dev"
echo ""
