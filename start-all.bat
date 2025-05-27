@echo off
REM Set legacy OpenSSL provider for Node.js
set NODE_OPTIONS=--openssl-legacy-provider

REM Start the backend server
start "Minecraft Backend" "%~dp0backend.exe"

REM Start the frontend React app
cd /d "%~dp0frontend"
start "Minecraft Frontend" cmd /k "set NODE_OPTIONS=--openssl-legacy-provider && npm start -- --host 0.0.0.0"

REM Optional: Go back to root
cd /d "%~dp0"

echo Both backend and frontend have been started in new windows.
pause