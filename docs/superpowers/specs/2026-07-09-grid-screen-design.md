# Grid Screen — Design Spec

> Window layout management tool for Linux (X11) and Windows.
> Drag a window over a pre-defined zone, release, and the window snaps into place.
> Multi-monitor support with auto-switching layouts.

## Overview

Grid Screen is a cross-platform desktop app that lets users define zones on their screen and snap application windows into those zones by dragging. It runs silently in the system tray and opens a GUI only for configuring layouts.

**Target users:** General, non-technical users. GUI-first, install-and-use experience.

### Key Features

- Drag window into zone → window snaps to zone size and position
- Zones defined on a resizable grid (default) with manual edge dragging
- Visual feedback during drag: zone borders + ghost window preview
- Multiple saved layouts, switchable manually or auto-activated by monitor hotplug
- Multi-monitor: independent zones per monitor, drag between monitors
- System tray app: always ready, "pause" toggle, auto-start option

## Technology Stack

| Layer | Technology |
|-------|-----------|
| App framework | Tauri 2.x |
| Backend | Rust |
| Frontend GUI | Web (HTML/CSS/JS, Svelte 5) |
| Windows platform API | `windows` crate (Win32) |
| Linux X11 API | `x11rb` crate |
| Linux Wayland (future) | `wayland-client` + `wayland-protocols-wlr` |
| Overlay rendering | `tiny-skia` |
| Config storage | JSON file in OS app data directory |

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  TRAY ICON                       │
│         (click -> open config window)            │
└──────────────┬──────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────┐
│            RUST BACKEND (always running)         │
│                                                  │
│  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │ Drag Detector│  │ Zone       │  │ Monitor  │ │
│  │ (window drag │  │ Overlay    │  │ Manager  │ │
│  │  events)     │  │ (visuals)  │  │ (display │ │
│  │              │  │            │  │  hotplug)│ │
│  └──────────────┘  └────────────┘  └──────────┘ │
│                                                  │
│  ┌──────────────┐  ┌────────────────────────────┐│
│  │ Layout       │  │  Platform Abstraction       ││
│  │ Manager      │  │  ┌─────────┐ ┌───────────┐ ││
│  │ (save/load/  │  │  │Windows  │ │Linux X11  │ ││
│  │  activate)   │  │  │Win32    │ │+ Wayland  │ ││
│  └──────────────┘  │  └─────────┘ └───────────┘ ││
│                    └────────────────────────────┘│
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │         Config Store (JSON file)          │   │
│  └──────────────────────────────────────────┘   │
└──────────────┬──────────────────────────────────┘
               │  Tauri IPC (invoke + events)
┌──────────────▼──────────────────────────────────┐
│          WEB FRONTEND (config window only)       │
│                                                  │
│  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │Layout Editor │  │Layout List │  │ Settings │ │
│  │(edit zones)  │  │(manage     │  │(autostart│ │
│  │              │  │ layouts)   │  │ UI opts) │ │
│  └──────────────┘  └────────────┘  └──────────┘ │
└─────────────────────────────────────────────────┘
```

**Principle:** Backend handles all real-time operations (drag detection, overlay rendering, window manipulation). Frontend is only for configuration — it opens on demand and does not participate in snap operations.

## Core Components

### Backend (Rust) — 7 modules

#### 1. PlatformApi (trait)

Unified interface for OS-level operations. Each OS provides its own implementation.

```
trait PlatformApi {
    fn enumerate_monitors()  -> Vec<Monitor>;
    fn enumerate_windows()   -> Vec<Window>;
    fn move_window(handle, rect);
    fn get_cursor_pos()      -> (i32, i32);
    fn subscribe_window_move_events(callback);
    fn create_overlay_window(monitor_id) -> OverlayHandle;
    fn destroy_overlay_window(handle);
}
```

Monitor struct: `{ id, name, x, y, width, height, dpi_scale, is_primary }`

#### 2. MonitorManager

- Calls `PlatformApi::enumerate_monitors()` on startup and every 2 seconds (polling)
- Computes a fingerprint: hash of (monitor count, each monitor's resolution and position)
- Emits internal event when fingerprint changes (hotplug detected)
- Provides `get_monitor_at(x, y)` for cursor-to-monitor lookup

#### 3. LayoutManager

- Owns all layout state in memory and on disk
- Layout = `HashMap<MonitorFingerprint, Vec<Zone>>`
- Zone = `{ id, name, x, y, width, height, gap, margin }`
- `activate_layout(fingerprint)`: switch active zones to match the given fingerprint
- `save_layout(name, layout)`: persist a named layout to disk
- `list_layouts()`: return all saved layouts with their monitor fingerprints
- On hotplug: looks up fingerprint → activates matching layout → falls back to default if none found

#### 4. DragDetector

- Subscribes to `PlatformApi::subscribe_window_move_events`
- On drag start: records window handle and original size, tells ZoneOverlay to show
- During drag: polls cursor position each frame, determines which monitor and zone the cursor is in, tells ZoneOverlay to update highlight and ghost preview
- On drag end / release: if cursor is inside a zone, calls `PlatformApi::move_window()` to snap; otherwise hides overlay and does nothing

#### 5. ZoneOverlay

- Creates transparent overlay windows (one per monitor) via `PlatformApi`
- Hidden by default, shown on drag start, hidden on drag end
- Renders: zone borders (2px, accent color), highlighted zone fill (20% accent), ghost window preview (translucent rectangle showing snapped size)
- Uses `tiny-skia` to draw into the overlay window buffer

#### 6. TrayManager

- System tray icon (Tauri tray API)
- Menu: "Configure" (opens config window), "Pause/Resume" (toggles snapping), "Quit"
- Pause state changes tray icon to indicate inactive status

#### 7. ConfigStore

- Reads/writes `layouts.json` at:
  - Linux: `~/.config/grid-screen/layouts.json`
  - Windows: `%APPDATA%/GridScreen/layouts.json`
- Validates JSON on load, falls back to default layout on corruption
- Creates backup before each write
- Handles schema migrations if format changes

### Frontend (Web) — 3 screens

#### Layout Editor

- Shows a scaled-down visual representation of current monitor layout
- Grid overlay on each monitor
- Operations: drag zone borders to resize, double-click zone to rename, right-click to delete, drag to create new zone
- "Apply" button sends layout to backend immediately, "Save" persists it under a name

#### Layout Manager

- Lists all saved layouts with name and small thumbnail
- Rename, delete, duplicate layouts
- Assign layout to a specific monitor configuration fingerprint
- Set one layout as default

#### Settings

- Auto-start with system toggle
- Zone gap/margin defaults
- Accent color for overlays
- Language (Vietnamese / English)
- Check for updates, About

## Data Flow

### Startup

```
App starts
  -> Tray icon created
  -> ConfigStore loads layouts from disk
  -> MonitorManager enumerates displays, computes fingerprint
  -> LayoutManager activates matching layout (or default if none)
  -> DragDetector begins listening for window drag events
  -> Ready
```

### Window drag-and-drop

```
DragDetector receives "drag start"
  -> Records window handle + original size
  -> ZoneOverlay shows on all monitors

During drag (per frame):
  -> Get cursor position
  -> MonitorManager::get_monitor_at(cursor) -> monitor
  -> LayoutManager::get_zones(monitor) -> zones
  -> Test point-in-zone for each zone
  -> ZoneOverlay updates (highlight matched zone, ghost preview)

Mouse release:
  -> If cursor in a zone: PlatformApi::move_window(handle, zone_rect)
  -> If cursor outside all zones: do nothing
  -> ZoneOverlay hides
```

### Monitor hotplug

```
MonitorManager polling detects change
  -> Compute new fingerprint
  -> LayoutManager::activate_layout(new_fingerprint)
     -> Found: apply new zones for each display
     -> Not found: show tray notification suggesting user create a layout
```

### Config window open

```
User clicks "Configure" in tray
  -> Tauri opens secondary webview window
  -> Frontend calls IPC get_current_state()
     <- Returns: monitors, active layout, all saved layouts
  -> User edits layout in editor
  -> "Apply" -> IPC apply_layout(layout) -> backend updates memory
  -> "Save"  -> IPC save_layout(name, layout) -> ConfigStore writes JSON
  -> Close window: backend continues running, no state lost
```

## State Management

Backend state is centralized in one struct behind `Arc<Mutex<...>>`:

```rust
struct AppState {
    active_layout: Layout,
    saved_layouts: Vec<SavedLayout>,
    monitors: Vec<Monitor>,
    is_paused: bool,
    is_dragging: bool,
    drag_state: Option<DragState>,
}
```

All modules (DragDetector, MonitorManager, ZoneOverlay, TrayManager) share this state via `Arc<Mutex<AppState>>` and run on their own threads where needed.

Frontend state is a read-only snapshot fetched on config window open, kept in sync via Tauri events when the backend state changes.

## Platform-Specific Details

### Windows

| Operation | Win32 API |
|-----------|-----------|
| Enumerate monitors | `EnumDisplayMonitors` + `GetMonitorInfoW` |
| Cursor position | `GetCursorPos` |
| Move/resize window | `SetWindowPos` with `HWND_TOP` |
| Enumerate windows | `EnumWindows` |
| Detect window drag | `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)` on title bar class |
| Overlay window | `CreateWindowExW` with `WS_EX_LAYERED`, `WS_EX_TRANSPARENT`, `WS_EX_TOOLWINDOW` |
| Draw on overlay | `UpdateLayeredWindow` or Direct2D child window |

Crate: `windows` (official Microsoft Rust crate).

### Linux

#### X11

Crate: `x11rb` (pure Rust, async-safe).

| Operation | X11 API |
|-----------|---------|
| Enumerate monitors | `xrandr` extension or `Xinerama` |
| Move/resize window | `XMoveResizeWindow` |
| Detect window drag | `ConfigureNotify` via `SubstructureRedirectMask` |
| Overlay window | `override_redirect = true`, transparent background |

#### Wayland

Wayland restricts third-party apps from manipulating other windows. Strategy:

- **Phase 1:** Detect Wayland, fall back to XWayland windows only. Show notification about native Wayland limitation.
- **Phase 2:** Use `ext-foreign-toplevel-list` protocol (supported by KDE, wlroots compositors). GNOME/Mutter requires separate extension.
- Crates: `wayland-client`, `wayland-protocols-wlr`

XWayland apps are treated as X11 windows and work fully on Phase 1.

## Error Handling

Guiding principle: the background app must never crash. All errors degrade gracefully.

| Scenario | Handling |
|----------|----------|
| Corrupted JSON config | Log error, fall back to default layout, do NOT overwrite corrupted file |
| Overlay window creation fails | Log error, disable overlay visuals, snapping still works |
| Wayland restricting window moves | Detect, show tray notification, disable snap for native Wayland windows |
| Monitor disconnected during drag | DragDetector cancels drag state, ZoneOverlay hides, no crash |
| Target window closed before snap | `move_window` fails, skip silently, debug log |
| Monitor polling transient error | Retry 3 times, keep last known monitor list if all fail |
| Cannot write config file | Log error, keep data in memory, retry after 5 minutes |

## Testing Strategy

| Type | Scope |
|------|-------|
| Rust unit tests | Zone geometry math (point-in-zone, overlap detection), fingerprint matching, JSON serialization/deserialization, config validation |
| Rust integration tests | Mock `PlatformApi` to test DragDetector logic, MonitorManager polling transitions, LayoutManager layout matching — no real display needed |
| Manual QA | Visual overlay rendering, drag-and-drop UX feel, multi-monitor hotplug, system tray behavior |
| No automated E2E | OS-level interaction is too fragile to automate; manual testing is sufficient for initial release |

## Constraints and Known Limitations

- Wayland native window snapping will not work in Phase 1 (XWayland apps will work)
- Requires users to create at least one layout before snapping works (default 1-zone-per-monitor layout is auto-created on first launch)
- Overlay rendering performance depends on OS compositor — may have slight latency on some Linux setups
- App requires system tray support (works on all major Linux DEs and Windows)

## Not In Scope (v1)

- Keyboard shortcuts for snap operations (drag-only in v1)
- Per-application zone assignment rules (e.g., "always put Chrome in zone 2")
- Touchscreen / tablet support
- macOS support
- Wayland native full support (Phase 2)
- Window grouping (snap multiple windows together)
