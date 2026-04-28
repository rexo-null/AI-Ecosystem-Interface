@echo off
REM ============================================================
REM ISKIN Run Script (Windows)
REM Запускает llama-server и ISKIN
REM ============================================================
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."

REM Значения по умолчанию
set "MODEL_PATH=models\qwen2.5-coder-14b-instruct-Q4_K_M.gguf"
set "HOST=127.0.0.1"
set "PORT=8080"
set "CTX_SIZE=8192"
set "GPU_LAYERS=45"
set "THREADS=8"
set "BATCH_SIZE=512"
set "UBATCH_SIZE=64"
set "CACHE_TYPE_K=q4_0"

echo [INFO] ISKIN Run - Windows
echo.

REM ============================================================
REM Поиск llama-server
REM ============================================================
set "LLAMA_SERVER="

where llama-server >nul 2>&1
if %errorlevel%==0 (
    set "LLAMA_SERVER=llama-server"
    echo [OK] llama-server найден в PATH
) else if exist "%PROJECT_DIR%\llama.cpp\build\bin\Release\llama-server.exe" (
    set "LLAMA_SERVER=%PROJECT_DIR%\llama.cpp\build\bin\Release\llama-server.exe"
    echo [OK] llama-server: !LLAMA_SERVER!
) else (
    echo [ERROR] llama-server не найден!
    echo Запустите scripts\setup.bat для установки.
    exit /b 1
)

REM ============================================================
REM Проверка модели
REM ============================================================
set "FULL_MODEL_PATH=%PROJECT_DIR%\%MODEL_PATH%"

if not exist "%FULL_MODEL_PATH%" (
    echo [ERROR] Модель не найдена: %FULL_MODEL_PATH%
    echo Запустите scripts\setup.bat для скачивания.
    exit /b 1
)

echo [OK] Модель: %MODEL_PATH%

REM ============================================================
REM Запуск llama-server
REM ============================================================
echo.
echo [INFO] Запуск llama-server...
echo   Host: %HOST%:%PORT%
echo   Context: %CTX_SIZE% tokens
echo   GPU Layers: %GPU_LAYERS%

start "llama-server" /B "%LLAMA_SERVER%" ^
    --model "%FULL_MODEL_PATH%" ^
    --ctx-size %CTX_SIZE% ^
    --n-gpu-layers %GPU_LAYERS% ^
    --threads %THREADS% ^
    --batch-size %BATCH_SIZE% ^
    --ubatch-size %UBATCH_SIZE% ^
    --host %HOST% ^
    --port %PORT% ^
    --cache-type-k %CACHE_TYPE_K% ^
    --flash-attn

REM Ожидание готовности
echo [INFO] Ожидание готовности llama-server...
set /a WAITED=0
set /a MAX_WAIT=60

:wait_loop
if %WAITED% geq %MAX_WAIT% goto wait_timeout
curl -s "http://%HOST%:%PORT%/v1/models" >nul 2>&1
if %errorlevel%==0 goto server_ready
timeout /t 2 /nobreak >nul
set /a WAITED+=2
echo|set /p="."
goto wait_loop

:wait_timeout
echo.
echo [ERROR] llama-server не запустился за %MAX_WAIT% секунд
exit /b 1

:server_ready
echo.
echo [OK] llama-server готов

REM ============================================================
REM Запуск ISKIN
REM ============================================================
echo.
echo [INFO] Запуск ISKIN...
cd /d "%PROJECT_DIR%"
npx tauri dev

endlocal
