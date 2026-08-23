@echo off
setlocal
echo Installing Siphonix...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
if %errorlevel% neq 0 (
    echo.
    echo Installation encountered an issue.
    pause
    exit /b %errorlevel%
)
echo.
pause
