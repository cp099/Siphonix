#!/usr/bin/env bash
set -euo pipefail

# Siphonix Standalone Installer for macOS & Linux
# Developed by: Chirag P Patil (cp099)
# https://github.com/cp099/Siphonix

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}"
cat << "EOF"
  ███████╗██╗██████╗ ██╗  ██╗ ██████╗ ███╗   ██╗██╗██╗  ██╗
  ██╔════╝██║██╔══██╗██║  ██║██╔═══██╗████╗  ██║██║╚██╗██╔╝
  ███████╗██║██████╔╝███████║██║   ██║██╔██╗ ██║██║ ╚███╔╝ 
  ╚════██║██║██╔═══╝ ██╔══██║██║   ██║██║╚██╗██║██║ ██╔██╗ 
  ███████║██║██║     ██║  ██║╚██████╔╝██║ ╚████║██║██╔╝ ██╗
  ╚══════╝╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝
EOF
echo -e "${NC}"
echo -e "   ${BOLD}Siphonix Installer${NC} — Universal Terminal Media Downloader"
echo -e "   Developed by: ${BOLD}Chirag P Patil${NC} (${CYAN}cp099${NC}) • ${CYAN}https://github.com/cp099${NC}"
echo ""

# 1. Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        PLATFORM="macos"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    *)
        echo -e "${RED}Error: Unsupported operating system: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="arm64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "  Platform detected: ${GREEN}${PLATFORM} (${TARGET_ARCH})${NC}"

# 2. Check for yt-dlp and ffmpeg runtime dependencies
MISSING_DEPS=()

if ! command -v yt-dlp >/dev/null 2>&1; then
    MISSING_DEPS+=("yt-dlp")
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
    MISSING_DEPS+=("ffmpeg")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "\n  ${YELLOW}${BOLD}Notice:${NC} Required runtime components are missing: ${MISSING_DEPS[*]}"
    if [ "$PLATFORM" = "macos" ]; then
        echo -e "  Install them using Homebrew:"
        echo -e "    ${CYAN}brew install yt-dlp ffmpeg${NC}\n"
    else
        echo -e "  Install them on Ubuntu/Debian:"
        echo -e "    ${CYAN}sudo apt update && sudo apt install ffmpeg && pip install yt-dlp${NC}\n"
        echo -e "  Or on Arch Linux:"
        echo -e "    ${CYAN}sudo pacman -S yt-dlp ffmpeg${NC}\n"
    fi
else
    echo -e "  Runtime dependencies: ${GREEN}yt-dlp and ffmpeg found${NC}"
fi

# 3. Determine Installation Destination
INSTALL_DIR="${SIPHONIX_INSTALL_DIR:-}"

if [ -z "$INSTALL_DIR" ]; then
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="/usr/local/bin"
    fi
fi

echo -e "  Target installation path: ${CYAN}${INSTALL_DIR}/siphonix${NC}"

# 4. Locate Binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_BINARY=""

if [ -f "${SCRIPT_DIR}/../target/release/siphonix" ]; then
    SOURCE_BINARY="${SCRIPT_DIR}/../target/release/siphonix"
elif [ -f "${SCRIPT_DIR}/siphonix" ]; then
    SOURCE_BINARY="${SCRIPT_DIR}/siphonix"
elif [ -f "${SCRIPT_DIR}/../siphonix" ]; then
    SOURCE_BINARY="${SCRIPT_DIR}/../siphonix"
elif command -v cargo >/dev/null 2>&1; then
    echo -e "  Building release binary with Cargo..."
    (cd "${SCRIPT_DIR}/.." && cargo build --release)
    SOURCE_BINARY="${SCRIPT_DIR}/../target/release/siphonix"
else
    echo -e "${RED}Error: Could not locate siphonix binary to install.${NC}"
    exit 1
fi

# 5. Copy Binary with Correct Permissions
if [ -w "$INSTALL_DIR" ]; then
    cp "$SOURCE_BINARY" "${INSTALL_DIR}/siphonix"
    chmod 755 "${INSTALL_DIR}/siphonix"
else
    echo -e "  Writing to ${INSTALL_DIR} requires administrative privileges (sudo):"
    sudo cp "$SOURCE_BINARY" "${INSTALL_DIR}/siphonix"
    sudo chmod 755 "${INSTALL_DIR}/siphonix"
fi

# 6. Verify PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo -e "\n  ${YELLOW}${BOLD}Warning:${NC} ${INSTALL_DIR} is not currently in your PATH."
    echo -e "  Add this line to your shell configuration (~/.zshrc or ~/.bashrc):"
    echo -e "    ${CYAN}export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}"
fi

# 7. Verification
echo -e "\n${GREEN}${BOLD}Installation Complete!${NC}"
if command -v siphonix >/dev/null 2>&1; then
    VERSION_OUTPUT="$(siphonix --version 2>/dev/null || echo "v0.1.0")"
    echo -e "  ${BOLD}${VERSION_OUTPUT}${NC} is ready to use."
    echo -e "  Run ${CYAN}siphonix${NC} to start the interactive wizard."
else
    echo -e "  Run ${CYAN}${INSTALL_DIR}/siphonix${NC} to start the interactive wizard."
fi
echo ""
