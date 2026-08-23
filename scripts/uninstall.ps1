# Siphonix Uninstaller for Windows
# Developed by: Chirag P Patil (cp099)
# https://github.com/cp099/Siphonix

$ErrorActionPreference = "Stop"

$installDir = "$env:LOCALAPPDATA\Programs\Siphonix"
if (Test-Path "$installDir\siphonix.exe") {
    Remove-Item -Path "$installDir\siphonix.exe" -Force
    Write-Host "Removed binary at $installDir\siphonix.exe" -ForegroundColor Green
}

# Clean User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -like "*$installDir*") {
    $pathParts = $userPath -split ';' | Where-Object { $_ -ne $installDir -and $_ -ne "" }
    $newPath = $pathParts -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "Cleaned $installDir from User PATH" -ForegroundColor Green
}

# Optional user data cleanup
$userDataDir = "$env:USERPROFILE\.siphonix"
if (Test-Path $userDataDir) {
    $confirm = Read-Host "Do you want to remove user data and history at $userDataDir? (y/N)"
    if ($confirm -eq 'y' -or $confirm -eq 'Y') {
        Remove-Item -Path $userDataDir -Recurse -Force
        Write-Host "Removed $userDataDir" -ForegroundColor Green
    }
}

Write-Host "`nUninstallation Complete!`n" -ForegroundColor Green
