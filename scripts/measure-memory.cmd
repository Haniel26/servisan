@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0measure-memory.ps1" %*
echo.
pause
