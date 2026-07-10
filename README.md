# Grid Screen

Cross-platform window zone management. Drag windows into predefined zones for instant snap-to-position.

**Platforms:** Linux (X11) · Windows

[![CI](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml/badge.svg)](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml)

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Development](#development)
- [Architecture](#architecture)
- [Configuration](#configuration)
- [Build & Distribution](#build--distribution)
- [Testing](#testing)
- [Contributing](#contributing)

---

## Features

| Feature | Description |
|---------|-------------|
| **Window Snap** | Drag any window into a zone → auto snap to position |
| **WYSIWYG Editor** | Visual drag-and-drop zone editor with 12-column grid snapping |
| **Multi-Monitor** | Hotplug-aware — auto detects new/removed displays, per-monitor layouts |
| **System Tray** | Runs in tray with menu: Configure / Pause / View Logs / Quit |
| **Visual Feedback** | Zone highlight + ghost preview when dragging a window |
| **Multilingual** | English + Vietnamese UI (svelte-i18n) |
| **First-Run Guide** | Onboarding overlay on first launch |
| **Error Notifications** | Toast notifications for backend errors (config corruption, save failures...) |
| **Auto Updates** | Tauri updater — automatically fetches new versions from GitHub Releases |

---

## Installation

Grid Screen supports two installation methods: **one-command script** (recommended) or **build from source**.

---

### Method 1: One-Command Script (Recommended)

The script automatically detects your OS, installs dependencies, builds, and configures autostart.

#### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh | bash
```

The script performs these steps automatically:
1. Detects and installs system dependencies (libgtk-3, libwebkit2gtk, libx11...)
2. Installs Rust (via rustup, if missing)
3. Clones the repository to `~/.local/share/grid-screen`
4. Builds the release binary (`cargo build --release --features custom-protocol`)
5. Copies the binary to `~/.local/bin/grid-screen`
6. Creates a `.desktop` file in `~/.config/autostart/` for auto-launch
7. Creates a desktop entry in `~/.local/share/applications/` for app menu

After installation, restart your session (or run `grid-screen` from terminal) to begin.

**Advanced install options:**

```bash
# Custom install directory
INSTALL_DIR=/opt/grid-screen bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)

# Development build (with debug logging)
INSTALL_MODE=dev bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)

# Dependencies only, skip build (for developers)
INSTALL_MODE=deps bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)
```

#### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.ps1 | iex
```

The PowerShell script will:
1. Install Rust via rustup-init.exe (if missing)
2. Install Node.js via winget (if missing)
3. Clone the repository to `%LOCALAPPDATA%\grid-screen`
4. Build the release binary
5. Copy to `%LOCALAPPDATA%\Programs\grid-screen\`
6. Add to registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` for autostart

---

### Method 2: Build from Source

For developers who want to customize or contribute.

#### System Requirements

| Component | Version |
|-----------|---------|
| Rust | stable (1.75+) |
| Node.js | 20+ |
| npm | 9+ |
| Git | 2.0+ |

**Linux — system libraries:**

```bash
# Ubuntu / Debian
sudo apt update
sudo apt install -y \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libx11-dev libxrandr-dev libxinerama-dev \
  libappindicator3-dev librsvg2-dev libdbus-1-dev \
  pkg-config libssl-dev

# Fedora
sudo dnf install -y \
  gtk3-devel webkit2gtk4.1-devel \
  libX11-devel libXrandr-devel libXinerama-devel \
  libappindicator-gtk3-devel librsvg2-devel dbus-devel \
  pkg-config openssl-devel

# Arch
sudo pacman -S --needed \
  gtk3 webkit2gtk-4.1 \
  libx11 libxrandr libxinerama \
  libappindicator-gtk3 librsvg dbus \
  pkg-config openssl
```

**Windows:** No extra libraries needed. Ensure [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) are installed with the "Desktop development with C++" workload.

#### Clone & Build

```bash
git clone https://github.com/enolalabs/grid-screen.git
cd grid-screen

# Install frontend dependencies
npm install

# Build
cargo tauri build
```

Output binary location:
- **Linux:** `src-tauri/target/release/grid-screen`
- **Windows:** `src-tauri/target/release/grid-screen.exe`

#### Development Mode

```bash
cargo tauri dev      # Hot-reload frontend + backend
```

#### Manual Binary Install

```bash
# Linux
sudo cp src-tauri/target/release/grid-screen /usr/local/bin/
mkdir -p ~/.config/autostart
cp assets/grid-screen.desktop ~/.config/autostart/

# Windows
# Copy grid-screen.exe anywhere, create shortcut in Startup folder
# (Win+R → shell:startup → paste shortcut)
```

---

### Uninstall

```bash
# Linux — if installed via script
~/.local/share/grid-screen/scripts/uninstall.sh

# Linux — if installed manually
rm /usr/local/bin/grid-screen
rm ~/.config/autostart/grid-screen.desktop
rm -rf ~/.config/grid-screen

# Windows — if installed via script
# Run "Uninstall Grid Screen" from Start Menu

# Windows — if installed manually
# Remove binary + Startup shortcut + registry key
reg delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v grid-screen
rmdir /s %LOCALAPPDATA%\grid-screen
```

---

## Usage

### First Launch

After installation, Grid Screen runs in the system tray.

1. **Right-click** the Grid Screen tray icon
2. Select **Configure** to open the configuration window
3. An **onboarding overlay** will appear — click **Got it** to begin

### System Tray Menu

| Item | Function |
|------|----------|
| **Configure** | Open config window (Editor, Layouts, Settings) |
| **Pause / Resume** | Toggle window snap functionality |
| **View Logs** | Open log file for debugging |
| **Quit** | Exit Grid Screen completely |

### Creating Your First Layout

1. Open the **Editor** tab in the config window
2. Each connected display appears as a scaled-down canvas
3. **Click on a monitor canvas** to create a new zone (defaults to 30% area)
4. **Zone operations:**
   - **Double-click** a zone → rename
   - **Right-click** a zone → delete confirmation dialog
   - **Arrow keys** → move zone (Shift for fine movement)
   - **Ctrl + Arrow keys** → resize zone
   - **Delete** key → delete focused zone
5. Zones automatically **snap to a 12-column grid** on create and move
6. Click **Apply Live** to activate the layout immediately — start dragging windows
7. Click **Save** to persist the layout with a name for later reuse

### Dragging Windows

Once a layout is applied:

1. **Drag any window** (File Explorer, Terminal, Browser...)
2. As you drag over a zone, it **highlights** (2px purple border + 20% fill)
3. A **ghost preview** shows where the window will snap
4. **Release the mouse** when the cursor is inside a zone → window auto-snaps
5. If you release outside all zones → the window stays where it is, unchanged

### Managing Layouts

- **Layouts** tab: view saved layouts
  - **Set Default** — make this layout load automatically on startup
  - **Delete** — remove the layout
- **Settings** tab:
  - **Auto-start** — launch with system
  - **Default gap** — spacing between adjacent zones (0–100px)
  - **Default margin** — offset from screen edges (0–100px)
  - **Accent color** — zone highlight and border color
  - **Language** — English / Tiếng Việt

### Keyboard Shortcuts

| Key | Mode | Action |
|-----|------|--------|
| **Arrow keys** | Editor — focused zone | Move zone |
| **Shift + Arrow** | Editor — focused zone | Fine move (1% of screen) |
| **Ctrl + Arrow** | Editor — focused zone | Resize zone |
| **Delete** | Editor — focused zone | Delete zone |

---

## Development

### Directory Structure

```
grid-screen/
├── src-tauri/                    # Rust backend (Tauri 2.x)
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri config (CSP, tray, bundler, updater)
│   ├── capabilities/
│   │   └── gridscreen.json       # Deny-by-default IPC permissions
│   ├── build.rs                  # Tauri build script
│   ├── icons/                    # Tray & app icons
│   ├── tests/                    # Integration tests
│   │   ├── config_store_tests.rs
│   │   ├── monitor_manager_tests.rs
│   │   ├── layout_manager_tests.rs
│   │   ├── zone_overlay_tests.rs
│   │   ├── drag_detector_tests.rs
│   │   └── security_smoke.rs
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri setup, IPC commands, thread wiring
│       ├── types.rs              # Shared data types (Monitor, Zone, Layout...)
│       ├── app_state.rs          # AppState with ArcSwap/RwLock/Mutex
│       ├── config_store.rs       # JSON config read/write + validation + backup rotation
│       ├── monitor_manager.rs    # Event-driven monitor detection + 30s safety-net polling
│       ├── layout_manager.rs     # Fractional coordinate math, zone operations
│       ├── drag_detector.rs      # Event-driven drag processor (blocking mpsc)
│       ├── zone_overlay.rs       # Transparent overlay rendering (tiny-skia)
│       ├── user_notifier.rs      # Backend → frontend error bridging
│       ├── perf.rs               # Frame counter & FPS stats
│       └── platform/
│           ├── mod.rs            # PlatformApi trait + conditional exports
│           ├── mock.rs           # Mock implementation (for testing)
│           ├── linux.rs          # X11 implementation (x11rb, RandR, Xinerama)
│           └── windows.rs        # Windows implementation (EnumDisplayMonitors, SetWindowPos...)
├── src/                          # Frontend (Svelte 5 + TypeScript)
│   ├── main.ts                   # App mount point
│   ├── App.svelte                # App shell (tabs, onboarding, toast)
│   ├── vite-env.d.ts             # Vite type declarations
│   ├── lib/
│   │   ├── types.ts              # TypeScript types mirroring Rust types
│   │   ├── ipc.ts                # Tauri invoke wrappers
│   │   ├── stores.ts             # Svelte stores (currentState, layouts...)
│   │   ├── notifications.ts      # Toast notification store
│   │   ├── i18n.ts               # i18n initialization
│   │   └── i18n/
│   │       ├── en.json           # English strings
│   │       └── vi.json           # Vietnamese strings
│   └── routes/
│       ├── LayoutEditor.svelte   # WYSIWYG zone editor (ARIA + keyboard a11y)
│       ├── LayoutManager.svelte  # Saved layouts list
│       ├── Settings.svelte       # Settings form
│       └── __tests__/
│           └── LayoutEditor.test.ts  # Component + keyboard nav tests
├── benches/
│   └── overlay_bench.rs          # Criterion benchmark (zone hit-test 64 zones)
├── scripts/
│   ├── install.sh                # Linux one-command installer
│   ├── install.ps1               # Windows one-command installer
│   └── uninstall.sh              # Linux uninstaller
├── .github/workflows/
│   └── ci.yml                    # CI matrix: ubuntu-latest + windows-latest
├── vitest.config.ts              # Vitest config (jsdom environment)
├── package.json                  # Node dependencies
└── svelte.config.js              # Svelte compiler config
```

### Dev Commands

```bash
# Install dependencies
npm install

# Run Tauri dev mode (hot-reload frontend + backend)
cargo tauri dev
```

### Useful Commands

```bash
# Backend
cargo fmt --check          # Check Rust formatting
cargo clippy -- -D warnings # Lint Rust
cargo test                  # Run all Rust tests
cargo bench                 # Run benchmarks
cargo build --release --features custom-protocol # Production build

# Frontend
npx vitest run              # Run component tests
npm run build               # Production Svelte build

# Security
cargo audit                 # Check dependency CVEs
cargo deny check            # Check licenses + banned deps
```

---

## Architecture

### Threading Model

Grid Screen uses a 4-thread model:

| Thread | Role | Sync Mechanism |
|--------|------|----------------|
| **Main** | Tauri runtime + system tray + IPC | — |
| **Platform event loop** | X11/Win32 event polling (window move, display change) | `mpsc::channel` → DragDetector |
| **Drag processor** | Receive events, hit-test zones, send snaps, control overlay | Blocking `mpsc::recv()` |
| **Overlay renderer** | Draw zone highlights + ghost rects via tiny-skia | Called from DragDetector callbacks |

### State Access

| Data | Mechanism | Rationale |
|------|-----------|-----------|
| `monitors` | `ArcSwap<Vec<Monitor>>` | Lock-free reads — hotpath |
| `active_layouts` | `ArcSwap<Vec<Layout>>` | Lock-free reads — hotpath |
| `drag_state` | `Mutex<Option<DragState>>` | Accessed only from DragDetector thread |
| `app_config` | `RwLock<AppConfig>` | Many readers, few writers (settings page) |

### Platform API

The backend uses a `PlatformApi` trait to abstract OS-level APIs:

```rust
pub trait PlatformApi: Send + Sync {
    fn enumerate_monitors(&self) -> Vec<Monitor>;
    fn enumerate_windows(&self) -> Vec<Window>;
    fn move_window(&self, handle: WindowHandle, rect: Rect);
    fn get_cursor_pos(&self) -> (i32, i32);
    fn is_mouse_button_down(&self) -> bool;
    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent>;
    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent>;
    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String>;
    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32);
    fn destroy_overlay_window(&self, handle: OverlayHandle);
    fn set_autostart(&self, enabled: bool) -> Result<(), String>;
}
```

| Implementation | Platform | Library |
|----------------|----------|---------|
| `LinuxPlatformApi` | X11 | `x11rb` (RandR, Xinerama, Shape, ConfigureWindow) |
| `WindowsPlatformApi` | Windows | `windows` crate (EnumDisplayMonitors, SetWindowPos...) |
| `MockPlatformApi` | Testing | OS-independent, used in unit/integration tests |

### Drag Flow

```
User drags a window
  → Platform API captures WindowMoveEvent
    → DragDetector receives event via blocking mpsc::recv()
      → DragStart: get cursor position, find monitor, show overlay
      → DragMove: hit-test zones (O(n), max 64), update highlight + ghost
      → DragEnd: if cursor is in a zone → send SnapEvent
        → Snap consumer thread calls platform_api.move_window()
```

### Fractional Coordinates

All zones are stored as fractional coordinates (0.0–1.0) relative to the monitor dimensions:

```
zone.x = 0.0, zone.width = 0.5  → left half of the screen (any resolution)
zone.y = 0.5, zone.height = 0.5 → bottom half of the screen
```

Converted to pixels at runtime:
```
pixel_x = monitor.x + zone.x * monitor.width + margin + gap/2
pixel_w = zone.width * monitor.width - 2*margin - gap
```

---

## Configuration

### Config File

Grid Screen stores configuration at `$XDG_CONFIG_HOME/grid-screen/layouts.json` (Linux) or `%APPDATA%/grid-screen/layouts.json` (Windows).

```json
{
  "schema_version": 1,
  "layouts": [
    {
      "id": "uuid",
      "name": "My Layout",
      "arrangement_id": "monitor_hash",
      "monitor_id": "uuid",
      "zones": [
        {
          "id": "uuid",
          "name": "Left Half",
          "x": 0.0, "y": 0.0,
          "width": 0.5, "height": 1.0,
          "gap": 4, "margin": 8
        }
      ]
    }
  ],
  "settings": {
    "auto_start": false,
    "default_gap": 4,
    "default_margin": 8,
    "accent_color": "#7C3AED",
    "language": "en",
    "first_run_completed": false,
    "default_layout_id": null
  }
}
```

### Validation

- Schema version check
- Zone/layout names: 1–64 characters
- Max 64 zones per monitor
- Coordinates in [0.0, 1.0] range
- No zone overlaps
- HTML escape zone names before saving (XSS prevention)
- Backup rotation: keeps 5 most recent `.bak.N` files
- Atomic write: write to `.tmp` → verify → rename

### Logging

Logs stored at `$XDG_CONFIG_HOME/grid-screen/grid-screen.log`:
- Daily rotation
- Max 3 files, each ≤ 1MB
- Panic hook automatically writes backtrace to log

---

## Build & Distribution

### Local Build

```bash
npm install
cargo tauri build
```

### CI Pipeline

GitHub Actions (`ubuntu-latest` + `windows-latest` matrix):
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo audit`
5. `cargo deny check`
6. `npm ci && npx vitest run`
7. `cargo build --release --features custom-protocol`

### Distribution Formats

| Platform | Format |
|----------|--------|
| Linux | `.deb` (Debian/Ubuntu), `.AppImage` (universal) |
| Windows | `.msi` (system-wide), NSIS `.exe` (per-user) |

### Auto-Update

Uses the Tauri updater plugin, checking GitHub Releases:

```
GET https://github.com/enolalabs/grid-screen/releases/latest/download/latest.json
```

### Security

- **Capabilities**: deny-by-default — only `core:default`, `tray:default`, and basic window permissions. No `shell:`, `http:`, `fs:`.
- **CSP**: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self' ipc: https://ipc.localhost; img-src 'self' data:;`
- **File permissions**: config file set to `0o600` (owner read/write only) on Linux
- **HTML escaping**: zone names escaped before storage (XSS prevention)
- **Smoke tests**: `security_smoke.rs` verifies capabilities and CSP at commit time

---

## Testing

### Unit & Integration Tests (Rust)

```bash
cargo test                                    # All tests
cargo test config_store                       # ConfigStore (6 tests)
cargo test monitor_manager                    # MonitorManager (2 tests)
cargo test layout_manager                     # LayoutManager (3 tests)
cargo test drag_detector                      # DragDetector (2 tests)
cargo test zone_overlay                       # ZoneOverlay (3 tests)
cargo test security_smoke                     # Security (2 tests)
```

### Component Tests (Svelte)

```bash
npx vitest run                                # LayoutEditor component + keyboard a11y
```

### Benchmarks

```bash
cargo bench                                   # Zone hit-test 64 zones @ 4K
```

Budget: `hit_test_64_zones` < 1ms (fits within 16ms frame budget even at max zones).

---

## Contributing

1. Fork the repository
2. Create a branch: `git checkout -b feature/your-feature`
3. Code + test: `cargo test && npx vitest run`
4. Format: `cargo fmt && cargo clippy -- -D warnings`
5. Commit with a clear message
6. Push and create a Pull Request

**Commit convention:** [Conventional Commits](https://www.conventionalcommits.org/)
- `feat:` — new feature
- `fix:` — bug fix
- `test:` — add/update tests
- `chore:` — config, build, CI
- `docs:` — documentation

### References

- [Design spec](docs/superpowers/specs/2026-07-09-grid-screen-design.md)
- [Implementation plan](docs/superpowers/plans/2026-07-09-grid-screen-implementation.md)
- [Tauri 2.x docs](https://v2.tauri.app/)
- [Svelte 5 docs](https://svelte.dev/docs)
- [x11rb docs](https://docs.rs/x11rb)

### Roadmap

| Phase | Scope |
|-------|-------|
| **v0.1** (current) | X11 backend, Windows backend, Svelte 5 config UI, tray |
| **v0.2** | Windows event hook refinement, pixel format conversion for overlay |
| **v0.3** | Wayland support (via XWayland), keyboard shortcuts |
| **v1.0** | macOS support, auto-layout suggestions |

---

## License

MIT
