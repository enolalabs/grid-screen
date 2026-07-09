#!/usr/bin/env bash
set -euo pipefail

# Grid Screen — Linux Install Script
# Usage: curl -fsSL https://.../install.sh | bash
# Options: INSTALL_DIR=/custom/path INSTALL_MODE=dev|deps|release bash install.sh

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/share/grid-screen}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
INSTALL_MODE="${INSTALL_MODE:-release}"
REPO_URL="https://github.com/enolalabs/grid-screen.git"

# ── Colors ──────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; NC='\033[0m'
info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}   $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Banner ──────────────────────────────────────
echo ""
echo "  ╔══════════════════════════════════════╗"
echo "  ║       Grid Screen Installer          ║"
echo "  ║   Cross-platform window zone manager ║"
echo "  ╚══════════════════════════════════════╝"
echo ""

# ── Detect OS ───────────────────────────────────
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    err "This script is for Linux only. For Windows, use install.ps1"
fi

# Detect package manager
if command -v apt &>/dev/null; then
    PKG_MGR="apt"
elif command -v dnf &>/dev/null; then
    PKG_MGR="dnf"
elif command -v pacman &>/dev/null; then
    PKG_MGR="pacman"
elif command -v zypper &>/dev/null; then
    PKG_MGR="zypper"
else
    warn "Unknown package manager. You may need to install dependencies manually."
    PKG_MGR="unknown"
fi

info "Detected: $OSTYPE, package manager: $PKG_MGR"
info "Install dir: $INSTALL_DIR"
info "Binary dir:  $BIN_DIR"
info "Mode:        $INSTALL_MODE"

# ── Step 1: System Dependencies ──────────────────
echo ""
info "Step 1/7: Installing system dependencies..."

install_deps() {
    case "$PKG_MGR" in
        apt)
            sudo apt update -qq
            sudo apt install -y -qq \
                build-essential curl git pkg-config libssl-dev \
                libgtk-3-dev libwebkit2gtk-4.1-dev \
                libx11-dev libxrandr-dev libxinerama-dev \
                libappindicator3-dev librsvg2-dev libdbus-1-dev
            ;;
        dnf)
            sudo dnf install -y \
                gcc gcc-c++ curl git pkg-config openssl-devel \
                gtk3-devel webkit2gtk4.1-devel \
                libX11-devel libXrandr-devel libXinerama-devel \
                libappindicator-gtk3-devel librsvg2-devel dbus-devel
            ;;
        pacman)
            sudo pacman -S --needed --noconfirm \
                base-devel curl git pkg-config openssl \
                gtk3 webkit2gtk-4.1 \
                libx11 libxrandr libxinerama \
                libappindicator-gtk3 librsvg dbus
            ;;
        *)
            warn "Please install these packages manually:"
            echo "  - gcc, git, curl, pkg-config, openssl-dev"
            echo "  - gtk3-dev, webkit2gtk-4.1-dev"
            echo "  - libx11-dev, libxrandr-dev, libxinerama-dev"
            echo "  - libappindicator-dev, librsvg-dev, libdbus-dev"
            ;;
    esac
}
install_deps
ok "System dependencies installed"

# ── Step 2: Rust ─────────────────────────────────
echo ""
info "Step 2/7: Checking Rust toolchain..."

if command -v rustc &>/dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    ok "Rust already installed: $RUST_VERSION"
else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
    ok "Rust installed: $(rustc --version)"
fi

# ── Step 3: Node.js ──────────────────────────────
echo ""
info "Step 3/7: Checking Node.js..."

if command -v node &>/dev/null; then
    NODE_VERSION=$(node --version)
    ok "Node.js already installed: $NODE_VERSION"
else
    warn "Node.js 20+ is required for building the frontend"
    warn "Install it via: https://nodejs.org/ or your package manager"
    warn "Then re-run this script"
    if [[ "$INSTALL_MODE" == "deps" ]]; then
        ok "Deps-only mode — skipping Node.js requirement"
    else
        err "Node.js not found. Install Node.js 20+ and re-run."
    fi
fi

# ── Step 4: Clone Repository ─────────────────────
echo ""
info "Step 4/7: Cloning repository..."

if [[ -d "$INSTALL_DIR/.git" ]]; then
    info "Repository exists, pulling latest..."
    git -C "$INSTALL_DIR" pull --ff-only origin main
    ok "Repository updated"
else
    git clone --depth 1 "$REPO_URL" "$INSTALL_DIR"
    ok "Repository cloned to $INSTALL_DIR"
fi

# ── Step 5: Build ────────────────────────────────
if [[ "$INSTALL_MODE" == "deps" ]]; then
    echo ""
    info "Deps-only mode — skipping build."
    echo ""
    echo "  To build manually:"
    echo "    cd $INSTALL_DIR"
    echo "    npm install"
    echo "    npm run build"
    echo "    cargo build --release --manifest-path src-tauri/Cargo.toml"
    echo ""
    ok "Installation complete (deps only)!"
    exit 0
fi

echo ""
info "Step 5/7: Installing frontend dependencies..."
cd "$INSTALL_DIR"
npm install --silent 2>&1 | tail -3
ok "Frontend dependencies installed"

echo ""
info "Step 6/7: Building Grid Screen (this may take 5-15 minutes)..."

# Build frontend first (needed for tauri embed)
info "Building frontend..."
npm run build --silent 2>&1 | tail -3

if [[ "$INSTALL_MODE" == "dev" ]]; then
    info "Building Rust (debug mode)..."
    cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
    BINARY_PATH="$INSTALL_DIR/src-tauri/target/debug/grid-screen"
else
    info "Building Rust (release mode)..."
    npx tauri build --ci 2>&1 | tail -10
    BINARY_PATH="$INSTALL_DIR/src-tauri/target/release/grid-screen"
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    err "Build failed. Check logs above for errors."
fi
ok "Build complete: $BINARY_PATH"

# ── Step 7: Install Binary ───────────────────────
echo ""
info "Step 7/7: Installing binary and integrations..."

mkdir -p "$BIN_DIR"
cp "$BINARY_PATH" "$BIN_DIR/grid-screen"
chmod +x "$BIN_DIR/grid-screen"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    SHELL_RC=""
    case "$SHELL" in
        */bash) SHELL_RC="$HOME/.bashrc" ;;
        */zsh)  SHELL_RC="$HOME/.zshrc" ;;
        */fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
    esac
    if [[ -n "$SHELL_RC" ]] && ! grep -q "$BIN_DIR" "$SHELL_RC" 2>/dev/null; then
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_RC"
        info "Added $BIN_DIR to PATH in $SHELL_RC"
    fi
fi

ok "Binary installed to $BIN_DIR/grid-screen"

# ── Desktop Entry ────────────────────────────────
DESKTOP_DIR="$HOME/.local/share/applications"
AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$DESKTOP_DIR" "$AUTOSTART_DIR"

DESKTOP_FILE="[Desktop Entry]
Type=Application
Name=Grid Screen
Comment=Window zone management
Exec=$BIN_DIR/grid-screen
Icon=$INSTALL_DIR/src-tauri/icons/icon.png
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true"

echo "$DESKTOP_FILE" > "$DESKTOP_DIR/grid-screen.desktop"
echo "$DESKTOP_FILE" > "$AUTOSTART_DIR/grid-screen.desktop"
chmod +x "$DESKTOP_DIR/grid-screen.desktop" "$AUTOSTART_DIR/grid-screen.desktop"

ok "Desktop entry created"
ok "Autostart enabled"

# ── Done ─────────────────────────────────────────
echo ""
echo "  ╔══════════════════════════════════════╗"
echo "  ║      Installation Complete!          ║"
echo "  ╚══════════════════════════════════════╝"
echo ""
echo "  Grid Screen installed to: $BIN_DIR/grid-screen"
echo ""
echo "  Start now:"
echo "    grid-screen"
echo ""
echo "  Or restart your session for autostart to take effect."
echo ""
echo "  Config:     ~/.config/grid-screen/"
echo "  Logs:       ~/.config/grid-screen/grid-screen.log"
echo "  Source:     $INSTALL_DIR"
echo ""
echo "  Uninstall:  $INSTALL_DIR/scripts/uninstall.sh"
echo ""
