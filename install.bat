@echo off
REM ISKIN - Intelligent Self-Improving Knowledge Interface Network
REM Installation Script for Windows

echo 🚀 Starting ISKIN Installation for Windows...
echo.

REM Check Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed. Please install Node.js 18+ from https://nodejs.org/
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo [SUCCESS] Node.js %NODE_VERSION% found

REM Check npm
where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] npm is not installed
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo [SUCCESS] npm %NPM_VERSION% found

REM Check Rust
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [INFO] Rust is not installed. Installing Rust...
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    REM Need to restart command prompt after Rust installation
    echo [WARNING] Please restart Command Prompt and run this script again
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo [SUCCESS] Rust %RUST_VERSION% found

REM Check Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Cargo is not installed
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo [SUCCESS] Cargo %CARGO_VERSION% found

REM Check Visual Studio Build Tools (required for Tauri on Windows)
echo [INFO] Checking for Visual Studio Build Tools...
where cl >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARNING] Visual Studio Build Tools not found.
    echo Please install "Desktop development with C++" workload from Visual Studio Installer
    echo Download: https://visualstudio.microsoft.com/downloads/
)

echo [INFO] Installing npm dependencies...
call npm install
if %errorlevel% neq 0 (
    echo [ERROR] Failed to install npm dependencies
    pause
    exit /b 1
)
echo [SUCCESS] npm dependencies installed

echo [INFO] Setting up Rust targets...
rustup target add x86_64-pc-windows-msvc 2>nul

echo [INFO] Checking Rust dependencies...
cd src-tauri
cargo check --quiet 2>nul
if %errorlevel% neq 0 (
    echo [WARNING] Some Rust dependencies may need additional setup. This is normal for first run.
) else (
    echo [SUCCESS] Rust dependencies are ready
)
cd ..

echo [INFO] Setting up models directory...
if not exist models mkdir models

echo # ISKIN Models Directory > models\README.md
echo. >> models\README.md
echo This directory will contain local AI models for ISKIN. >> models\README.md
echo. >> models\README.md
echo ## Planned Models: >> models\README.md
echo - Qwen-2.5-Coder-14B (GGUF format) >> models\README.md
echo - Qwen-VL (for vision capabilities) >> models\README.md
echo - Custom fine-tuned models >> models\README.md
echo. >> models\README.md
echo ## Download Instructions: >> models\README.md
echo Models will be downloaded automatically by the application when first used, >> models\README.md
echo or you can manually download them from Hugging Face. >> models\README.md
echo. >> models\README.md
echo For now, ISKIN uses API-based models. Local model support will be added in Phase 3. >> models\README.md

echo [SUCCESS] Models directory created

echo [INFO] Creating launch scripts...

REM Create development launch script
echo @echo off > launch-dev.bat
echo REM ISKIN Development Launcher >> launch-dev.bat
echo. >> launch-dev.bat
echo echo Starting ISKIN Development Environment... >> launch-dev.bat
echo. >> launch-dev.bat
echo echo Starting React frontend... >> launch-dev.bat
echo start /B npm run dev >> launch-dev.bat
echo. >> launch-dev.bat
echo timeout /t 3 /nobreak ^> nul >> launch-dev.bat
echo. >> launch-dev.bat
echo echo Starting Tauri development... >> launch-dev.bat
echo cd src-tauri ^&^& cargo tauri dev >> launch-dev.bat

REM Create production build script
echo @echo off > build-release.bat
echo REM ISKIN Release Build Script >> build-release.bat
echo. >> build-release.bat
echo echo Building ISKIN for release... >> build-release.bat
echo. >> build-release.bat
echo echo Building React frontend... >> build-release.bat
echo call npm run build >> build-release.bat
echo. >> build-release.bat
echo echo Building Tauri application... >> build-release.bat
echo cd src-tauri ^&^& cargo tauri build --release >> build-release.bat
echo. >> build-release.bat
echo echo Build complete! Check src-tauri\target\release\ for the executable. >> build-release.bat
echo pause >> build-release.bat

echo [SUCCESS] Launch scripts created

echo.
echo ✅ ISKIN installation completed for Windows!
echo.
echo To start development:
echo   Run: launch-dev.bat
echo.
echo To build for release:
echo   Run: build-release.bat
echo.
echo Happy coding with ISKIN! 🤖
echo.
pause