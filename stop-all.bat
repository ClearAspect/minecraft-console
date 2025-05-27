@echo off
REM Kill the backend process (backend.exe)
taskkill /IM backend.exe /F

REM Kill the frontend process (node.exe, used by npm start)
taskkill /IM node.exe /F

REM Optionally, also kill any npm processes (if you want)
taskkill /IM npm.exe /F

echo Both backend and frontend have been stopped.
pause