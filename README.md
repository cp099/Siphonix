# Siphonix

```text
  ███████╗██╗██████╗ ██╗  ██╗ ██████╗ ███╗   ██╗██╗██╗  ██╗
  ██╔════╝██║██╔══██╗██║  ██║██╔═══██╗████╗  ██║██║╚██╗██╔╝
  ███████╗██║██████╔╝███████║██║   ██║██╔██╗ ██║██║ ╚███╔╝ 
  ╚════██║██║██╔═══╝ ██╔══██║██║   ██║██║╚██╗██║██║ ██╔██╗ 
  ███████║██║██║     ██║  ██║╚██████╔╝██║ ╚████║██║██╔╝ ██╗
  ╚══════╝╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝

   Siphonix v0.1.0 — Universal Terminal Media Downloader
   Developed by: Chirag P Patil (cp099) • https://github.com/cp099
```

[![Release](https://img.shields.io/github/v/release/cp099/Siphonix?style=flat-square&color=cyan)](https://github.com/cp099/Siphonix/releases/tag/v0.1.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg?style=flat-square)]()

> **Universal High-Performance Terminal Media Downloader** — Fast, robust, and reliable video and audio extraction powered by Rust, `yt-dlp`, and `FFmpeg`.

---

## 🌟 Overview

**Siphonix** is a modern terminal-native media downloader engineered for maximum speed, universal hardware playback compatibility, and extreme reliability. Whether launched as an interactive numbered terminal wizard or executed through direct CLI commands, Siphonix guarantees flawless downloads from YouTube with zero configuration headaches.

---

## ✨ Key Highlights

- **Universal QuickTime & Apple Silicon Playback**: Video downloads in MP4 format are automatically multiplexed with high-bitrate **H.264 (`avc1`)** video and **320 kbps AAC (`mp4a`)** audio, guaranteeing 100% native compatibility with macOS QuickTime Player, iOS Photos, Windows Media Player, and VLC.
- **Studio-Quality Audio Extraction**: Audio downloads extract pristine **320 kbps MP3** or lossless **24-bit FLAC** directly from best-quality source audio streams (`ba/b`) with embedded metadata ID3 tags and album cover art.
- **Interactive 4-Screen Numbered Wizard (TUI)**: A clean, distraction-free terminal wizard navigated with number keys (`1`–`7`) + Enter.
- **Direct CLI Automation**: Full command-line flag parser (`clap`) for automated scripts, cron jobs, and power users.
- **Real-Time Streaming Progress**: Live progress bars displaying download percentage, download speed (`MiB/s`), ETA countdown, and FFmpeg multiplexing stages.
- **Download History & SQLite Persistence**: High-concurrency WAL-mode SQLite database tracking download provenance, file sizes, timestamps, and on-disk verification (`siphonix --history`).
- **Zero-Orphan Process Management**: Subprocesses are isolated with `kill_on_drop(true)`. On `Ctrl+C`, active child processes are immediately killed and partial `.part` / `.ytdl` staging files are wiped cleanly.
- **System & Engine Diagnostics**: One-command diagnostic verification of installed engine binaries and runtime health (`siphonix --diagnostics`).

---

## 📋 Prerequisites

Siphonix requires `yt-dlp` and `ffmpeg` to be installed and available on your system `PATH`:

### macOS (Homebrew)
```bash
brew install yt-dlp ffmpeg
```

### Windows (Winget / Chocolatey)
```powershell
# Using Windows Package Manager (winget)
winget install yt-dlp Gyan.FFmpeg

# Or using Chocolatey
choco install yt-dlp ffmpeg
```

### Linux (Ubuntu / Debian / Arch)
```bash
# Ubuntu / Debian
sudo apt update && sudo apt install ffmpeg python3-pip
pip install --upgrade yt-dlp

# Arch Linux
sudo pacman -S yt-dlp ffmpeg
```

---

## 🚀 Installation

### Option 1: Quick Install (Prebuilt Release Archive)

#### macOS & Linux:
```bash
# Download and install latest release
curl -sSL https://github.com/cp099/Siphonix/releases/download/v0.1.0/siphonix-v0.1.0-darwin-arm64.tar.gz | tar -xz
./siphonix-v0.1.0-darwin-arm64/install.sh
```

#### Windows (PowerShell / Direct ZIP):
1. Download **[`siphonix-v0.1.0-windows-x86_64.zip`](https://github.com/cp099/Siphonix/releases/download/v0.1.0/siphonix-v0.1.0-windows-x86_64.zip)**.
2. Extract the ZIP file.
3. Right-click `install.ps1` and select **"Run with PowerShell"** (or run `powershell -ExecutionPolicy Bypass -File .\install.ps1`).

---

### Option 2: Build from Source with Cargo

```bash
# Clone the repository
git clone https://github.com/cp099/Siphonix.git
cd Siphonix

# Build and install globally to ~/.cargo/bin
cargo install --path . --force
```

Ensure `~/.cargo/bin` is in your shell `PATH`. You can now run `siphonix` from any terminal!

---

## 🎮 Usage

### 1. Interactive Wizard Mode

Simply run `siphonix` with no arguments to launch the 4-step interactive wizard:

```bash
siphonix
```

1. **Main Menu / URL Input**: Paste any YouTube video or playlist URL (or select options `1`–`4`).
2. **Metadata Card**: Displays video title, duration, and uploader.
3. **Format Selection**: Choose target format via number keys:
   - `[1]` MP4 Video (Universal QuickTime / Apple Silicon / Windows)
   - `[2]` MP3 Audio (Universal / 320 kbps)
   - `[3]` MKV Video (High Fidelity / Multi-track)
   - `[4]` FLAC Audio (Lossless Studio Quality)
   - `[5]` M4A Audio (Apple AAC)
4. **Quality Selection**:
   - For Video: `Best Available`, `4K (2160p)`, `2K (1440p)`, `1080p (Full HD)`, `720p (HD)`, `480p (SD)`, `360p (Low)`.
   - For Audio: `320 kbps (High Quality)`, `256 kbps`, `192 kbps`, `128 kbps`, `Best VBR`.
5. **Destination**: Select download folder (defaults to `~/Downloads/Siphonix`).
6. **Live Progress Terminal**: Real-time progress bar with speed, ETA, and post-download actions (open enclosing folder, play file, or download another).

---

### 2. Direct CLI Commands

For fast downloads, shell scripting, or headless servers:

```bash
# Download MP4 Video in 1080p Full HD
siphonix "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -m video -f MP4 -q 1080p

# Download MP4 Video in 4K (2160p)
siphonix "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -m video -f MP4 -q 2160p

# Download MP3 Audio in 320 kbps (Studio Quality)
siphonix "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -m audio -f MP3 -q 320k

# Download Lossless FLAC Audio
siphonix "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -m audio -f FLAC

# Custom Output Destination
siphonix "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -o ~/Desktop/Music -m audio -f MP3

# View Download History & Provenance
siphonix --history

# Run Engine & System Health Diagnostics
siphonix --diagnostics
```

---

## ⚙️ CLI Options Reference

```text
Usage: siphonix [OPTIONS] [URL]

Arguments:
  [URL]  URL to download (video or playlist)

Options:
  -m, --mode <MODE>        Media mode: video or audio [possible values: video, audio]
  -f, --format <FORMAT>    Target format (mp4, mkv, webm, mp3, flac, m4a, wav, opus)
  -q, --quality <QUALITY>  Target quality (2160p, 1440p, 1080p, 720p, 480p, 360p, best, 320k, 256k, 192k, 128k)
  -o, --output <OUTPUT>    Output destination folder [default: ~/Downloads/Siphonix]
      --embed-thumbnail    Embed thumbnail cover art into output file [default: true]
      --embed-metadata     Embed metadata ID3/MP4 tags into output file [default: true]
  -l, --history            View download history from local SQLite library
  -d, --diagnostics        Run engine & system dependency diagnostics
  -h, --help               Print help
  -V, --version            Print version
```

---

## 🏗️ Project Architecture

```text
Siphonix/
├── Cargo.toml                  # Root manifest & dependencies
├── LICENSE                     # MIT License
├── README.md                   # Complete documentation
├── RELEASE_NOTES.md            # Official v0.1.0 release notes
├── DEPLOYMENT.md               # Cross-platform deployment manual
├── dist/                       # Standalone release bundles & checksums
├── scripts/                    # Automated deployment & packaging scripts
│   ├── install.sh              # POSIX installer for macOS & Linux
│   ├── uninstall.sh            # POSIX uninstaller
│   ├── install.ps1             # PowerShell installer for Windows
│   ├── uninstall.ps1           # PowerShell uninstaller for Windows
│   └── package_release.sh      # Automated release packager
├── migrations/                 # Embedded SQLite schema migrations
│   ├── 0001_initial_schema.sql
│   ├── 0002_add_playlist_provenance.sql
│   ├── 0003_add_library_items.sql
│   └── 0004_add_download_options.sql
├── src/
│   ├── main.rs                 # Pure async entry point
│   ├── lib.rs                  # Crate root & module exports
│   ├── error.rs                # Unified error handling (thiserror)
│   ├── cli/                    # Clap parser, banner, direct downloader
│   ├── tui/                    # 4-screen interactive numbered wizard
│   ├── engine/                 # Command builder, detector, options, runner
│   ├── queue/                  # Job state machine, backoff, cooldown, scheduler
│   ├── db/                     # High-concurrency WAL SQLite repository
│   ├── library/                # Media provenance, disk verification, OS opener
│   ├── diagnostics/            # Health monitoring, non-blocking logger
│   └── runtime/                # Binary discovery, version checker
└── tests/                      # 72 automated unit & integration test suites
```

---

## 🤝 Open Source Acknowledgements

Siphonix is built with gratitude on top of the following open source tools and libraries:

| Project | Description | GitHub Repository |
| :--- | :--- | :--- |
| **yt-dlp** | Core command-line audio/video extraction engine | [https://github.com/yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) |
| **FFmpeg** | Multimedia processing, stream muxing, and transcoding framework | [https://github.com/FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg) |
| **Tokio** | Asynchronous runtime for the Rust programming language | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| **SQLx** | Async, pure Rust SQL crate with compile-time checked queries | [https://github.com/launchbadge/sqlx](https://github.com/launchbadge/sqlx) |
| **Clap** | Command Line Argument Parser for Rust | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| **Inquire** | Interactive terminal prompting and select menus for Rust | [https://github.com/mikaelmello/inquire](https://github.com/mikaelmello/inquire) |
| **Indicatif** | Progress reporting and terminal indicator library for Rust | [https://github.com/console-rs/indicatif](https://github.com/console-rs/indicatif) |
| **Colored** | Terminal string coloring and styling for ANSI output | [https://github.com/mackwic/colored](https://github.com/mackwic/colored) |
| **Chrono** | Date and time handling for Rust | [https://github.com/chronotope/chrono](https://github.com/chronotope/chrono) |
| **Serde / Serde JSON** | Serialization and deserialization framework for Rust | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| **Thiserror** | Convenient error derive macro for structured Rust errors | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| **Opener** | Cross-platform OS file and directory opener | [https://github.com/vorner/opener](https://github.com/vorner/opener) |
| **Regex** | Regular expressions for Rust with safe compile-time semantics | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |

---

## 👤 Author

Developed by **Chirag P Patil** ([@cp099](https://github.com/cp099))
- **GitHub Profile**: [https://github.com/cp099](https://github.com/cp099)
- **Project Repository**: [https://github.com/cp099/Siphonix](https://github.com/cp099/Siphonix)
- **Latest Release**: [https://github.com/cp099/Siphonix/releases/tag/v0.1.0](https://github.com/cp099/Siphonix/releases/tag/v0.1.0)

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.
