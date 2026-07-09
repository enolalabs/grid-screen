# Grid Screen

Cross-platform window zone management. Drag windows into pre-defined zones for instant positioning.

**Works on:** Linux (X11) · Windows

[![CI](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml/badge.svg)](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml)

## Features

- Drag windows into zones → instant snap
- WYSIWYG zone editor with grid snapping
- Multi-monitor with hotplug-aware layout switching
- System tray app with pause toggle
- Visual feedback: zone highlights + ghost window preview

## Dev Setup

**Prerequisites:** Rust stable, Node.js 20+

**Linux:** `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libx11-dev libxrandr-dev libxinerama-dev`

```bash
cargo tauri dev
```

## Architecture

| Layer | Stack |
|-------|-------|
| App framework | Tauri 2.x |
| Backend | Rust |
| Frontend | Svelte 5 |
| Windows API | `windows` crate |
| Linux API | `x11rb` |
| Rendering | `tiny-skia` |

See [design spec](docs/superpowers/specs/2026-07-09-grid-screen-design.md) for details.
