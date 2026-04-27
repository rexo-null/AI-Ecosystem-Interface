#!/bin/bash

# ISKIN Quick Start Script
# Starts all services and opens the application

echo "🚀 Starting ISKIN..."

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Start frontend
echo "Starting React frontend..."
npm run dev &
FRONTEND_PID=$!

# Wait for frontend to start
sleep 5

# Try to start Tauri (will fail gracefully if system deps missing)
echo "Attempting to start Tauri application..."
cd src-tauri
if cargo tauri dev 2>/dev/null; then
    echo "Tauri started successfully"
else
    echo "Tauri failed to start (likely missing system dependencies)"
    echo "Run './install.sh' to install all dependencies"
    echo "Frontend is running at http://localhost:1420"
fi

cd ..

# Cleanup
kill $FRONTEND_PID 2>/dev/null || true