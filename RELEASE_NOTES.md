# Siphonix v0.1.0 — Official Production Release Notes

**Release Date**: August 23, 2026  
**Author**: Chirag P Patil ([@cp099](https://github.com/cp099))  
**Repository**: [https://github.com/cp099/Siphonix](https://github.com/cp099/Siphonix)  
**License**: MIT License  

---

## What's New in v0.1.0

Siphonix v0.1.0 is the initial production-ready release of the **Universal High-Performance Terminal Media Downloader**, written in 100% safe Rust and powered by `yt-dlp` and `FFmpeg`.

### 🚀 Highlights & Features

1. **Universal Hardware & QuickTime Compatibility**:
   - MP4 video streams are automatically multiplexed with high-bitrate **H.264 (`avc1`)** video and **320 kbps AAC (`mp4a`)** audio.
   - Guaranteed native playback out-of-the-box on macOS QuickTime Player, iOS Photos, Windows Media Player, and VLC with zero format errors or black-screen artifacts.

2. **Studio-Quality Audio Extraction**:
   - Extract pristine **320 kbps MP3** or lossless **24-bit FLAC** directly from source audio streams.
   - Automatically embeds full ID3/MP4 metadata tags (Title, Artist, Genre, Date, Lyrics) and high-resolution album cover art into output files.

3. **Interactive 4-Screen Numbered Terminal UI**:
   - Distraction-free, clean ASCII terminal wizard navigated with simple number keys (`1`–`7`) + Enter.
   - Real-time streaming progress bars displaying percentage, speed in `MiB/s`, ETA countdown, and multiplexing stage indicators.

4. **Direct CLI Interface**:
   - Full command-line flag parser (`clap`) for automated scripts, cron jobs, and power users (`siphonix <URL> -m video -f MP4 -q 1080p`).

5. **Local SQLite Persistence & History**:
   - High-throughput WAL-mode SQLite database tracking download provenance, file sizes, timestamps, and on-disk verification (`siphonix --history`).

6. **Production Hardened & Zero Resource Leaks**:
   - Subprocess lifecycle management with `kill_on_drop(true)`.
   - `Ctrl+C` interrupt handler immediately kills child processes and cleans up partial `.part` / `.ytdl` staging files.
   - 0 production `unwrap()` / `expect()` panics and 0 `unsafe` blocks.

7. **Automated Cross-Platform Installation**:
   - Turn-key POSIX (`install.sh`) and Windows PowerShell (`install.ps1`) scripts requiring zero development tools on end-user machines.

---

## 📦 Release Artifacts

| Platform / Architecture | Archive Name | SHA-256 Checksum |
| :--- | :--- | :--- |
| **macOS Apple Silicon (arm64)** | `siphonix-v0.1.0-darwin-arm64.tar.gz` | `eca87f5cc18877f129c7c23b5fa9a93f6332ee4d8638eada73d135e00208f380` |

---

## 🛠️ Quick Install

### macOS & Linux
```bash
tar -xzf siphonix-v0.1.0-*.tar.gz
cd siphonix-v0.1.0-*
./install.sh
```

### Windows
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```
