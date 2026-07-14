# Grid Screen

<div align="center">

**A lightning-fast window arranger for Linux X11. Drag windows into zones, click arrange.**

[![Rust](https://img.shields.io/badge/rust-1.96+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/svelte-5.0-ff3e00.svg)](https://svelte.dev)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

</div>

---

## What is Grid Screen?

Grid Screen lets you arrange your desktop windows into **zones** — predefined screen splits like two columns, three columns, or a main area with a sidebar. Instead of manually dragging and resizing every window, you **drag window cards onto a visual canvas** and click **Arrange**. All selected windows snap into place at once.

Built for **general desktop users** who want a tidy workspace without learning tiling window managers or memorizing keyboard shortcuts.

<img src=".github/screenshot.png" alt="Grid Screen Arrange view" width="100%">

## Features

- **5 built-in presets** — Two Columns, Three Columns, Focus + Stack, Main + Sidebar, 3 Wide Center
- **Drag-and-drop canvas** — assign windows to zones visually before applying
- **Customizable layouts** — adjust divider ratio, gap, and margin with sliders; save your own layouts
- **Multiple screens** — select which monitor to arrange
- **System tray** — runs in the background, open when needed
- **Zero network** — no telemetry, no analytics, no cloud. Everything stays on your machine
- **Lightweight** — idle CPU <1%, written in Rust

## Installation

### Download

Download the latest `.AppImage` or `.deb` from [GitHub Releases](https://github.com/enolalabs/grid-screen/releases).

**AppImage:**
```bash
chmod +x grid-screen_*.AppImage
./grid-screen_*.AppImage
```

**Debian/Ubuntu:**
```bash
sudo dpkg -i grid-screen_*.deb
```

### Requirements

- **Linux** with **X11** session (Xorg). Wayland with XWayland is partially supported.
- A window manager with **EWMH** support (GNOME, KDE Plasma, Xfce — most modern WMs work).

> **Wayland users:** Grid Screen requires X11. On XWayland, basic functionality works but full screen detection may be limited. Native Wayland support is planned.

## Usage

### Arrange Windows

1. Open Grid Screen. The **Arrange** tab shows your running windows on the left.
2. **Choose a screen** and **layout** from the toolbar.
3. **Drag window cards** from the catalog into the zones on the canvas.
4. Adjust the **divider ratio**, **gap**, and **margin** sliders if needed.
5. Click **Arrange N windows**. All assigned windows snap into their zones.

<img src=".github/usage-arrange.png" alt="Drag windows into zones" width="600">

### Create Custom Layouts

1. Go to the **Layouts** tab.
2. Click **Duplicate** on any preset to create a copy.
3. Give it a name and save.
4. Switch to Arrange, select your layout, adjust sliders. Changes auto-save on successful arrangement.

### Settings

- **Snap modifier & behavior** (coming soon)
- **Start at Login** — auto-launch Grid Screen when you log in
- **Minimize to Tray** — keep running when you close the window
- **System Status** — view session type, EWMH support, connected screens

## Development

### Prerequisites

- [Rust](https://rustup.rs) 1.96+
- [Node.js](https://nodejs.org) 20+
- Linux system dependencies for Tauri:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev \
    librsvg2-dev patchelf libx11-dev libxrandr-dev
  ```

### Setup

```bash
git clone https://github.com/enolalabs/grid-screen.git
cd grid-screen
npm install
```

### Run in development

```bash
npm run tauri dev
```

### Build for production

```bash
npm run tauri build
```

Produces `.deb` and `.AppImage` in `src-tauri/target/release/bundle/`.

### Project Structure

```
grid-screen/
├── src/                    # Svelte UI
│   ├── components/         # 25 UI components
│   ├── lib/
│   │   ├── stores/         # 8 Svelte stores
│   │   ├── commands.ts     # Tauri IPC wrapper
│   │   └── events.ts       # Rust → Svelte event listeners
│   └── App.svelte          # Root component with tab routing
├── src-tauri/              # Rust application core
│   └── src/
│       ├── main.rs         # Tauri entry point & lifecycle
│       ├── app_shell.rs    # Tauri commands (bootstrap, arrange, save)
│       ├── layout_engine.rs    # Zone rectangle computation
│       ├── arrange_orchestrator.rs  # Validate-then-move pipeline
│       ├── platform_adapter.rs     # OS abstraction trait
│       ├── x11_adapter.rs         # X11/EWMH/XRandR integration
│       ├── config_store.rs        # Atomic JSON persistence
│       ├── window_catalog.rs      # Window eligibility filtering
│       └── diagnostics.rs         # File-based logging
├── shared-types/           # Rust ↔ TypeScript type definitions
├── mockups/                # UI design mockups (dark, light, pro)
└── docs/                   # Design specs & implementation plans
```

### Architecture

```
Svelte UI (webview)
     ↕ Tauri IPC (typed commands + events)
Rust application core
     ↕ PlatformAdapter trait
X11 adapter (XRandR + EWMH)
```

The Rust core handles all X11 communication, layout computation, and config persistence. The Svelte webview handles only the UI — drag-and-drop state, canvas rendering, and user input. The webview can be destroyed (window closed) while the Rust core keeps running in the system tray.

### Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Application core | [Rust](https://www.rust-lang.org) |
| UI framework | [Svelte 5](https://svelte.dev) + [TypeScript](https://www.typescriptlang.org) |
| X11 integration | [x11rb](https://crates.io/crates/x11rb) |
| Config persistence | JSON in `~/.config/grid-screen/` |
| Logging | [tracing](https://crates.io/crates/tracing) |
| Drag-and-drop | Pointer-event-based (custom implementation) |

## Contributing

Contributions are welcome! Here's how to get started:

1. **Check the issues** — look for `good-first-issue` or `help-wanted` labels
2. **Read the docs** — design specs and implementation plans are in `docs/`
3. **Pick a task** — comment on the issue to claim it
4. **Follow the code style** — Rust: `cargo fmt` + `cargo clippy`; Svelte: `npm run lint`
5. **Write tests** — Rust: `cargo test`; UI: verify component renders correctly
6. **Open a PR** — keep it focused, reference the issue, include screenshots for UI changes

### Key areas for contributions

- **X11 adapter hardening** — real window enumeration, EWMH move/resize, edge cases
- **Wayland support** — implement `PlatformAdapter` for Wayland compositors
- **Modifier-assisted snap** — drag real system windows into zones with a modifier key
- **Accessibility** — keyboard navigation, screen reader support, ARIA
- **Testing** — integration tests, X11 test suite, property-based geometry tests

## License

MIT © [Enola Labs](https://github.com/enolalabs)
