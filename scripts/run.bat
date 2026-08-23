@echo off
title Siphonix - Universal Terminal Media Downloader
cd /d "%~dp0"
if exist "siphonix.exe" (
    siphonix.exe
) else if exist "%LOCALAPPDATA%\Programs\Siphonix\siphonix.exe" (
    "%LOCALAPPDATA%\Programs\Siphonix\siphonix.exe"
) else (
    siphonix
)
if %errorlevel% neq 0 (
    echo.
    echo Siphonix session ended.
    pause
)
