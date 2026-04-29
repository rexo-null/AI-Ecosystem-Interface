@echo off
REM ============================================================
REM ISKIN Setup Script (Windows)
REM Устанавливает все зависимости для запуска ISKIN с llama.cpp
REM ============================================================
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "MODELS_DIR=%PROJECT_DIR%\models"
set "LLAMA_DIR=%PROJECT_DIR%\llama.cpp"

echo [INFO] ISKIN Setup - Windows
echo.

REM ============================================================
REM 1. Проверка системных зависимостей
REM ============================================================
echo [INFO] Проверка системных зависимостей...

where rustc >nul 2>&1
if %errorlevel%==0 (
    for /f "tokens=*" %%i in ('rustc --version') do echo [OK] Rust: %%i
) else (
    echo [ERROR] Rust не найден.
    echo Установите Rust: https://rustup.rs/
    echo Запустите установщик и перезапустите терминал.
    exit /b 1
)

where node >nul 2>&1
if %errorlevel%==0 (
    for /f "tokens=*" %%i in ('node --version') do echo [OK] Node.js: %%i
) else (
    echo [ERROR] Node.js не найден. Установите Node.js 18+: https://nodejs.org/
    exit /b 1
)

where npm >nul 2>&1
if %errorlevel%==0 (
    echo [OK] npm найден
) else (
    echo [ERROR] npm не найден.
    exit /b 1
)

where git >nul 2>&1
if %errorlevel%==0 (
    echo [OK] Git найден
) else (
    echo [ERROR] Git не найден. Установите Git: https://git-scm.com/
    exit /b 1
)

where cmake >nul 2>&1
if %errorlevel%==0 (
    echo [OK] CMake найден
) else (
    echo [WARN] CMake не найден.
    echo Установите CMake: https://cmake.org/download/
    echo Или через winget: winget install Kitware.CMake
    exit /b 1
)

REM ============================================================
REM 2. Сборка llama.cpp
REM ============================================================
echo.
echo [INFO] Подготовка llama.cpp...

if exist "%LLAMA_DIR%\build\bin\Release\llama-server.exe" (
    echo [OK] llama.cpp уже собран
) else (
    if not exist "%LLAMA_DIR%" (
        echo [INFO] Клонирование llama.cpp...
        git clone https://github.com/ggerganov/llama.cpp.git "%LLAMA_DIR%"
    )

    cd /d "%LLAMA_DIR%"
    if not exist build mkdir build
    cd build

    echo [INFO] Сборка llama.cpp (это может занять несколько минут)...

    REM Попытка обнаружить CUDA
    where nvcc >nul 2>&1
    if %errorlevel%==0 (
        echo [INFO] Обнаружен NVIDIA GPU (CUDA)
        cmake .. -DGGML_CUDA=ON -DLLAMA_BUILD_SERVER=ON
    ) else (
        echo [INFO] GPU не обнаружен, сборка для CPU
        cmake .. -DLLAMA_BUILD_SERVER=ON
    )

    cmake --build . --config Release

    echo [OK] llama.cpp собран
    cd /d "%PROJECT_DIR%"
)

REM ============================================================
REM 3. Скачивание модели
REM ============================================================
echo.
set "MODEL_FILE=%MODELS_DIR%\qwen2.5-coder-14b-instruct-Q4_K_M.gguf"

echo [INFO] Проверка модели Qwen2.5-Coder-14B...

if exist "%MODEL_FILE%" (
    echo [OK] Модель уже скачана
) else (
    if not exist "%MODELS_DIR%" mkdir "%MODELS_DIR%"

    echo [INFO] Скачивание Qwen2.5-Coder-14B (Q4_K_M, ~9.2 GB)...
    echo [INFO] Это может занять значительное время.

    set "MODEL_URL=https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF/resolve/main/qwen2.5-coder-14b-instruct-q4_k_m.gguf"

    where curl >nul 2>&1
    if %errorlevel%==0 (
        curl -L -C - -o "%MODEL_FILE%" "!MODEL_URL!"
    ) else (
        echo [ERROR] curl не найден. Скачайте модель вручную:
        echo   URL: !MODEL_URL!
        echo   Путь: %MODEL_FILE%
        exit /b 1
    )

    echo [OK] Модель скачана
)

REM ============================================================
REM 4. Установка npm зависимостей
REM ============================================================
echo.
echo [INFO] Установка npm зависимостей...
cd /d "%PROJECT_DIR%"
call npm install
echo [OK] npm зависимости установлены

REM ============================================================
REM Готово
REM ============================================================
echo.
echo ======================================
echo   ISKIN Setup завершён успешно!
echo ======================================
echo.
echo Для запуска ISKIN выполните:
echo   scripts\run.bat
echo.

endlocal
