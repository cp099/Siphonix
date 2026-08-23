# Siphonix Production Deployment Guide

This guide covers deployment, distribution, runtime requirements, and setup of Siphonix on macOS, Windows, and Linux environments.

---

## 1. Distribution Architecture

Siphonix is compiled into a single, self-contained native binary. It has no dynamic C library runtime requirements beyond standard OS system libraries.

```text
Release Bundle (siphonix-v0.1.0-<platform>-<arch>.tar.gz)
├── siphonix            # Stripped standalone executable (~5-8 MB)
├── install.sh          # One-step automated installer
├── uninstall.sh        # Clean uninstaller
├── README.md           # User manual & CLI reference
└── LICENSE             # MIT License
```

---

## 2. Runtime Prerequisites

Siphonix acts as an orchestrator for stream extraction and hardware-accelerated multiplexing. It expects two CLI tools to be reachable on the system `PATH`:

| Tool | Minimum Version | Purpose |
| :--- | :--- | :--- |
| **`yt-dlp`** | `2023.03.04+` | Media metadata inspection, extraction, and stream downloading |
| **`ffmpeg`** | `4.4+` | Fast stream demuxing, AAC/H.264 muxing, metadata/cover art embedding |

### System Dependency Installation

#### macOS
```bash
# Using Homebrew (recommended)
brew install yt-dlp ffmpeg
```

#### Windows
```powershell
# Using Windows Package Manager (winget)
winget install yt-dlp Gyan.FFmpeg

# Or using Chocolatey
choco install yt-dlp ffmpeg
```

#### Linux
```bash
# Ubuntu / Debian
sudo apt update
sudo apt install ffmpeg python3-pip
pip install --upgrade yt-dlp

# Arch Linux / Manjaro
sudo pacman -S yt-dlp ffmpeg

# Fedora / RHEL
sudo dnf install ffmpeg yt-dlp
```

---

## 3. End-User Installation

### macOS & Linux

#### Method A: Using the Installer Script
Extract the release tarball and run `install.sh`:
```bash
tar -xzf siphonix-v0.1.0-*.tar.gz
cd siphonix-v0.1.0-*
./install.sh
```

#### Method B: Manual Binary Placement
```bash
sudo cp siphonix /usr/local/bin/siphonix
sudo chmod 755 /usr/local/bin/siphonix
```

---

### Windows

#### Method A: Using PowerShell Installer
Extract the release archive and in PowerShell run:
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

#### Method B: Manual Placement
1. Copy `siphonix.exe` to `%LOCALAPPDATA%\Programs\Siphonix\siphonix.exe`.
2. Add `%LOCALAPPDATA%\Programs\Siphonix` to your User `PATH` environment variable.

---

## 4. Filesystem Hierarchy & Default Paths

When Siphonix executes, it automatically provisions the following directories:

| Path | Purpose | Permissions |
| :--- | :--- | :--- |
| `~/.siphonix/` | Application data root directory | `0700` (User only) |
| `~/.siphonix/siphonix.db` | SQLite library, job history & provenance database | `0600` (User only) |
| `~/.siphonix/logs/` | Non-blocking rotating diagnostic logs | `0700` (User only) |
| `~/Downloads/Siphonix/` | Default media download destination directory | Standard user permissions |
| `/tmp/siphonix_temp/` (or OS temp) | Temporary segment extraction staging area | System temp |

---

## 5. Environment Variables & Overrides

Siphonix respects standard system environment variables and supports explicit binary overrides:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `SIPHONIX_YTDLP_PATH` | Explicit absolute path to `yt-dlp` executable | Discovered via `PATH` |
| `SIPHONIX_FFMPEG_PATH` | Explicit absolute path to `ffmpeg` executable | Discovered via `PATH` |
| `SIPHONIX_INSTALL_DIR` | Target installation directory for `install.sh` | `/usr/local/bin` or `~/.local/bin` |
| `HOME` (Unix) / `USERPROFILE` (Win) | User home directory used to anchor `~/.siphonix` | OS User Profile |

---

## 6. Verification & Health Check

After deployment, verify that Siphonix and its runtime dependencies are operating correctly:

```bash
# 1. Verify binary version and CLI execution
siphonix --version

# 2. Run automated diagnostic health check
siphonix --diagnostics
```

Expected diagnostic output:
```text
═════════════════════════════════════════════════════════════════
  Siphonix System & Engine Diagnostics
═════════════════════════════════════════════════════════════════

  ✓ yt-dlp binary:  /usr/local/bin/yt-dlp
  ✓ ffmpeg binary:  /usr/local/bin/ffmpeg
  ✓ yt-dlp version: 2026.08.20.234504
  ✓ ffmpeg version: ffmpeg version 8.1.1

  Engine Health: READY (All dependencies operational)
═════════════════════════════════════════════════════════════════
```

---

## 7. Uninstallation

To remove Siphonix cleanly from a machine:

### macOS / Linux
```bash
./scripts/uninstall.sh
```

### Windows
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1
```
