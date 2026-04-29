@echo off
REM ISKIN Quick Start Script for Windows

echo 🚀 Starting ISKIN...

REM Check if dependencies are installed
if not exist node_modules (
    echo Installing dependencies...
    call npm install
)

REM Start frontend
echo Starting React frontend...
start /B npm run dev

REM Wait for frontend to start
timeout /t 5 /nobreak > nul

REM Try to start Tauri
echo Attempting to start Tauri application...
cd src-tauri
cargo tauri dev 2>nul
if %errorlevel% neq 0 (
    echo Tauri failed to start (likely missing system dependencies)
    echo Run 'install.bat' to install all dependencies
    echo Frontend is running at http://localhost:1420
)

cd ..
pause