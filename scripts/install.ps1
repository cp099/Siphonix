# Siphonix Standalone Installer for Windows
# Developed by: Chirag P Patil (cp099)
# https://github.com/cp099/Siphonix

$ErrorActionPreference = "Stop"

Write-Host @"
  ███████╗██╗██████╗ ██╗  ██╗ ██████╗ ███╗   ██╗██╗██╗  ██╗
  ██╔════╝██║██╔══██╗██║  ██║██╔═══██╗████╗  ██║██║╚██╗██╔╝
  ███████╗██║██████╔╝███████║██║   ██║██╔██╗ ██║██║ ╚███╔╝ 
  ╚════██║██║██╔═══╝ ██╔══██║██║   ██║██║╚██╗██║██║ ██╔██╗ 
  ███████║██║██║     ██║  ██║╚██████╔╝██║ ╚████║██║██╔╝ ██╗
  ╚══════╝╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝
"@ -ForegroundColor Cyan

Write-Host "`n   Siphonix Installer — Universal Terminal Media Downloader" -ForegroundColor White
Write-Host "   Developed by: Chirag P Patil (cp099) • https://github.com/cp099`n" -ForegroundColor DarkGray

# 1. Runtime Dependencies Check
$missing = @()
if (-not (Get-Command "yt-dlp" -ErrorAction SilentlyContinue)) {
    $missing += "yt-dlp"
}
if (-not (Get-Command "ffmpeg" -ErrorAction SilentlyContinue)) {
    $missing += "ffmpeg"
}

if ($missing.Count -gt 0) {
    Write-Host "  Notice: Required runtime components are missing: $($missing -join ', ')" -ForegroundColor Yellow
    Write-Host "  Install them using winget:" -ForegroundColor White
    Write-Host "    winget install yt-dlp Gyan.FFmpeg`n" -ForegroundColor Cyan
} else {
    Write-Host "  Runtime dependencies: yt-dlp and ffmpeg found" -ForegroundColor Green
}

# 2. Setup Target Directory
$installDir = "$env:LOCALAPPDATA\Programs\Siphonix"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# 3. Locate Source Binary
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$sourceBinary = ""

if (Test-Path "$scriptDir\..\target\release\siphonix.exe") {
    $sourceBinary = "$scriptDir\..\target\release\siphonix.exe"
} elseif (Test-Path "$scriptDir\siphonix.exe") {
    $sourceBinary = "$scriptDir\siphonix.exe"
} elseif (Test-Path "$scriptDir\..\siphonix.exe") {
    $sourceBinary = "$scriptDir\..\siphonix.exe"
} else {
    Write-Host "Error: Could not locate siphonix.exe to install." -ForegroundColor Red
    Exit 1
}

# 4. Copy Binary
Copy-Item -Path $sourceBinary -Destination "$installDir\siphonix.exe" -Force
Write-Host "  Installed binary to: $installDir\siphonix.exe" -ForegroundColor Green

# 5. Update User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    $newPath = "$userPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "  Added $installDir to User PATH" -ForegroundColor Green
}

Write-Host "`nInstallation Complete!" -ForegroundColor Green
Write-Host "Run 'siphonix' in any new PowerShell or Terminal window to start.`n" -ForegroundColor White
