#!/usr/bin/env bash
set -euo pipefail

# Grid Screen — Uninstall Script

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; NC='\033[0m'
info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}   $*"; }

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/share/grid-screen}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

echo ""
echo "  Uninstalling Grid Screen..."
echo ""

read -p "  This will remove Grid Screen completely. Continue? [y/N] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "  Cancelled."
    exit 0
fi

# Remove binary
if [[ -f "$BIN_DIR/grid-screen" ]]; then
    rm "$BIN_DIR/grid-screen"
    ok "Removed $BIN_DIR/grid-screen"
fi

# Remove desktop entries
rm -f "$HOME/.local/share/applications/grid-screen.desktop"
rm -f "$HOME/.config/autostart/grid-screen.desktop"
ok "Removed desktop entries"

# Remove source directory
if [[ -d "$INSTALL_DIR" ]]; then
    rm -rf "$INSTALL_DIR"
    ok "Removed $INSTALL_DIR"
fi

# Remove config (ask)
CONFIG_DIR="$HOME/.config/grid-screen"
if [[ -d "$CONFIG_DIR" ]]; then
    read -p "  Remove config files at $CONFIG_DIR? [y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        ok "Removed $CONFIG_DIR"
    else
        info "Kept config at $CONFIG_DIR"
    fi
fi

echo ""
ok "Grid Screen uninstalled."
echo ""
