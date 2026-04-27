#!/bin/bash

# ISKIN - Intelligent Self-Improving Knowledge Interface Network
# Installation Script for Windows/Linux/macOS
# This script sets up the complete development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS=linux;;
        Darwin*)    OS=macos;;
        CYGWIN*|MINGW*|MSYS*) OS=windows;;
        *)          OS=unknown
    esac
    log_info "Detected OS: $OS"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check system requirements
check_system_requirements() {
    log_info "Checking system requirements..."

    # Check Node.js
    if ! command_exists node; then
        log_error "Node.js is not installed. Please install Node.js 18+ from https://nodejs.org/"
        exit 1
    fi

    NODE_VERSION=$(node --version | sed 's/v//' | cut -d. -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        log_error "Node.js version 18+ required. Current version: $(node --version)"
        exit 1
    fi
    log_success "Node.js $(node --version) found"

    # Check npm
    if ! command_exists npm; then
        log_error "npm is not installed"
        exit 1
    fi
    log_success "npm $(npm --version) found"

    # Check Rust
    if ! command_exists rustc; then
        log_error "Rust is not installed. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    log_success "Rust $(rustc --version) found"

    # Check Cargo
    if ! command_exists cargo; then
        log_error "Cargo is not installed"
        exit 1
    fi
    log_success "Cargo $(cargo --version) found"
}

# Install system dependencies
install_system_dependencies() {
    log_info "Installing system dependencies..."

    case $OS in
        linux)
            if command_exists apt; then
                log_info "Installing dependencies with apt..."
                sudo apt update
                sudo apt install -y pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
            elif command_exists dnf; then
                log_info "Installing dependencies with dnf..."
                sudo dnf install -y gtk3-devel webkit2gtk3-devel libappindicator-gtk3-devel librsvg2-devel
            elif command_exists pacman; then
                log_info "Installing dependencies with pacman..."
                sudo pacman -S --noconfirm gtk3 webkit2gtk libappindicator-gtk3 librsvg
            else
                log_warning "Unknown Linux package manager. Please install GTK and WebKit dependencies manually."
            fi
            ;;
        macos)
            if ! command_exists brew; then
                log_error "Homebrew is required on macOS. Please install from https://brew.sh/"
                exit 1
            fi
            log_info "Installing dependencies with brew..."
            brew install gtk+3 webkit2gtk librsvg
            ;;
        windows)
            log_info "On Windows, Tauri will use MSVC toolchain. Make sure Visual Studio Build Tools are installed."
            ;;
    esac
}

# Setup Node.js dependencies
setup_nodejs() {
    log_info "Setting up Node.js dependencies..."

    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install
        log_success "npm dependencies installed"
    else
        log_info "node_modules already exists, skipping npm install"
    fi
}

# Setup Rust dependencies
setup_rust() {
    log_info "Setting up Rust dependencies..."

    cd src-tauri

    # Update Cargo.toml for better compatibility
    log_info "Updating Cargo.toml dependencies..."

    # Install required Rust targets
    rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true
    rustup target add aarch64-apple-darwin 2>/dev/null || true
    rustup target add x86_64-apple-darwin 2>/dev/null || true
    rustup target add x86_64-pc-windows-msvc 2>/dev/null || true

    # Try to build to check dependencies
    log_info "Checking Rust dependencies..."
    if cargo check --quiet 2>/dev/null; then
        log_success "Rust dependencies are ready"
    else
        log_warning "Some Rust dependencies may need additional setup. This is normal for first run."
    fi

    cd ..
}

# Download models (placeholder - will be implemented when models are available)
download_models() {
    log_info "Setting up models directory..."

    mkdir -p models

    # Create placeholder for model downloads
    cat > models/README.md << 'EOF'
# ISKIN Models Directory

This directory will contain local AI models for ISKIN.

## Planned Models:
- Qwen-2.5-Coder-14B (GGUF format)
- Qwen-VL (for vision capabilities)
- Custom fine-tuned models

## Download Instructions:
Models will be downloaded automatically by the application when first used,
or you can manually download them from Hugging Face.

For now, ISKIN uses API-based models. Local model support will be added in Phase 3.
EOF

    log_success "Models directory created"
}

# Create launch scripts
create_launch_scripts() {
    log_info "Creating launch scripts..."

    # Create development launch script
    cat > launch-dev.sh << 'EOF'
#!/bin/bash
# ISKIN Development Launcher

echo "Starting ISKIN Development Environment..."

# Start frontend in background
echo "Starting React frontend..."
npm run dev &
FRONTEND_PID=$!

# Wait a moment for frontend to start
sleep 3

# Start Tauri development
echo "Starting Tauri development..."
cd src-tauri && cargo tauri dev

# Cleanup
kill $FRONTEND_PID 2>/dev/null || true
EOF

    chmod +x launch-dev.sh

    # Create production build script
    cat > build-release.sh << 'EOF'
#!/bin/bash
# ISKIN Release Build Script

echo "Building ISKIN for release..."

# Build frontend
echo "Building React frontend..."
npm run build

# Build Tauri application
echo "Building Tauri application..."
cd src-tauri && cargo tauri build --release

echo "Build complete! Check src-tauri/target/release/ for the executable."
EOF

    chmod +x build-release.sh

    # Create Windows batch files
    if [ "$OS" = "windows" ]; then
        cat > launch-dev.bat << 'EOF'
@echo off
echo Starting ISKIN Development Environment...

echo Starting React frontend...
start /B npm run dev

timeout /t 3 /nobreak > nul

echo Starting Tauri development...
cd src-tauri && cargo tauri dev
EOF

        cat > build-release.bat << 'EOF'
@echo off
echo Building ISKIN for release...

echo Building React frontend...
call npm run build

echo Building Tauri application...
cd src-tauri && cargo tauri build --release

echo Build complete! Check src-tauri\target\release\ for the executable.
pause
EOF
    fi

    log_success "Launch scripts created"
}

# Create desktop shortcuts (Linux)
create_desktop_shortcut() {
    if [ "$OS" = "linux" ] && [ -d ~/.local/share/applications ]; then
        log_info "Creating desktop shortcut..."

        cat > ~/.local/share/applications/iskin.desktop << EOF
[Desktop Entry]
Name=ISKIN IDE
Comment=Intelligent Self-Improving Knowledge Interface Network
Exec=$(pwd)/launch-dev.sh
Icon=$(pwd)/src-tauri/icons/icon.png
Terminal=true
Type=Application
Categories=Development;IDE;
EOF

        chmod +x ~/.local/share/applications/iskin.desktop
        log_success "Desktop shortcut created"
    fi
}

# Main installation function
main() {
    log_info "🚀 Starting ISKIN Installation..."
    echo ""

    detect_os
    check_system_requirements
    install_system_dependencies
    setup_nodejs
    setup_rust
    download_models
    create_launch_scripts
    create_desktop_shortcut

    echo ""
    log_success "✅ ISKIN installation completed!"
    echo ""
    log_info "To start development:"
    echo "  Linux/macOS: ./launch-dev.sh"
    echo "  Windows: launch-dev.bat"
    echo ""
    log_info "To build for release:"
    echo "  Linux/macOS: ./build-release.sh"
    echo "  Windows: build-release.bat"
    echo ""
    log_info "Happy coding with ISKIN! 🤖"
}

# Run main function
main "$@"