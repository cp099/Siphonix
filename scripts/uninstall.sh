#!/usr/bin/env bash
set -euo pipefail

# Siphonix Uninstaller for macOS & Linux
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
echo -e "   ${BOLD}Siphonix Uninstaller${NC}"
echo -e "   Developed by: ${BOLD}Chirag P Patil${NC} (${CYAN}cp099${NC}) • ${CYAN}https://github.com/cp099${NC}"
echo ""

POSSIBLE_PATHS=(
    "/usr/local/bin/siphonix"
    "$HOME/.local/bin/siphonix"
    "$HOME/.cargo/bin/siphonix"
)

FOUND=0

for BIN_PATH in "${POSSIBLE_PATHS[@]}"; do
    if [ -f "$BIN_PATH" ]; then
        echo -e "  Found binary at: ${CYAN}${BIN_PATH}${NC}"
        if [ -w "$(dirname "$BIN_PATH")" ]; then
            rm -f "$BIN_PATH"
        else
            sudo rm -f "$BIN_PATH"
        fi
        echo -e "  ${GREEN}Removed binary: ${BIN_PATH}${NC}"
        FOUND=1
    fi
done

if [ $FOUND -eq 0 ]; then
    echo -e "  ${YELLOW}No installed siphonix binary was found in standard locations.${NC}"
fi

# Option to remove user database and logs
USER_DATA_DIR="$HOME/.siphonix"
if [ -d "$USER_DATA_DIR" ]; then
    read -r -p "  Do you also want to remove user data and history at ~/.siphonix? [y/N]: " REMOVE_DATA
    if [[ "$REMOVE_DATA" =~ ^[Yy]$ ]]; then
        rm -rf "$USER_DATA_DIR"
        echo -e "  ${GREEN}Removed user data at ~/.siphonix${NC}"
    else
        echo -e "  ${CYAN}Kept user data at ~/.siphonix${NC}"
    fi
fi

echo -e "\n${GREEN}${BOLD}Uninstallation Complete!${NC}\n"
