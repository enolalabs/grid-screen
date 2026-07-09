# Grid Screen — Design Spec

> Window layout management tool for Linux (X11) and Windows.
> Drag a window over a pre-defined zone, release, and the window snaps into place.
> Multi-monitor support with auto-switching layouts.

## Problem Statement

Users with large or multiple monitors waste time manually resizing and repositioning application windows to use their screen space effectively. Existing solutions are either too technical (tiling window managers like i3, bspwm) or limited (Windows Snap, Linux half-screen shortcuts). Grid Screen provides a GUI-first, drag-and-drop zone system that works consistently across Windows and Linux for non-technical users.

**Target users:** General, non-technical users. GUI-first, install-and-use experience.

### Competitive Positioning

| Feature | Grid Screen | PowerToys FancyZones | i3/bspwm |
|---------|-------------|---------------------|----------|
| Setup UX | GUI drag-and-drop editor | GUI editor (Windows only) | Text config files |
| Cross-platform | Windows + Linux | Windows only | Linux only |
| Multi-monitor hotplug | Auto-switch layouts | Manual | Manual |
| Visual drag feedback | Zone highlights + ghost preview | Zone highlights | None |
| System tray / pause | Yes | No (always on) | N/A |

### Success Criteria (v1)

1. User can install, create a layout with 3+ zones, and snap a window within 2 minutes of first launch
2. Window snap completes within 200ms of mouse release (measured from release to window in final position)
3. Background CPU usage < 1% when idle, < 15% during active drag on a 4-monitor setup
4. Zero crashes from error paths — all failures degrade gracefully with user-visible feedback

## Key Features

- Drag window into zone → window snaps to zone size and position
- Zones defined on a configurable grid (default 12 columns) with manual edge dragging
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
| Error types | `thiserror` for domain errors |
| Logging | `tracing` crate, file appender with rotation |

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  TRAY ICON                       │
│         (click → open config window)            │
└──────────────┬──────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────┐
│            RUST BACKEND (always running)         │
│                                                  │
│  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │ Drag Detector│  │ Zone       │  │ Monitor  │ │
│  │ (event-      │  │ Overlay    │  │ Manager  │ │
│  │  driven)     │  │ (click-thru│  │ (native  │ │
│  │              │  │  visuals)  │  │  events) │ │
│  └──────────────┘  └────────────┘  └──────────┘ │
│                                                  │
│  ┌──────────────┐  ┌────────────────────────────┐│
│  │ Layout       │  │  Platform Abstraction       ││
│  │ Manager      │  │  ┌─────────┐ ┌───────────┐ ││
│  │ (fractional  │  │  │Windows  │ │Linux X11  │ ││
│  │  coords)     │  │  │Win32    │ │· ·W·l·d· ·│ ││
│  └──────────────┘  │  └─────────┘ └───────────┘ ││
│                    └────────────────────────────┘│
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │   Config Store (JSON + schema_version)    │   │
│  └──────────────────────────────────────────┘   │
└──────────────┬──────────────────────────────────┘
               │  Tauri IPC (capability-restricted)
┌──────────────▼──────────────────────────────────┐
│          WEB FRONTEND (config window only)       │
│          CSP: strict, no remote content          │
│                                                  │
│  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │Layout Editor │  │Layout List │  │ Settings │ │
│  │(WYSIWYG grid)│  │(manage     │  │(autostart│ │
│  │              │  │ layouts)   │  │ UI opts) │ │
│  └──────────────┘  └────────────┘  └──────────┘ │
└─────────────────────────────────────────────────┘
```

**Principle:** Backend handles all real-time operations (drag detection, overlay rendering, window manipulation). Frontend is only for configuration — it opens on demand and does not participate in snap operations.

## Core Components

### Backend (Rust) — 7 modules

#### 1. PlatformApi (trait)

Unified interface for OS-level operations. Each OS provides its own implementation.
All platform interaction is isolated behind this trait and never exposes OS handles to other modules.

```rust
trait PlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor>;
    fn enumerate_windows(&self) -> Vec<Window>;
    fn move_window(&self, handle: WindowHandle, rect: Rect);
    fn get_cursor_pos(&self) -> (i32, i32);
    fn is_mouse_button_down(&self) -> bool;

    // Returns a channel receiver. The impl spawns a dedicated event-loop thread
    // and sends WindowMoveEvent through the channel. Dropping the receiver
    // signals the impl to tear down the thread and unregister hooks.
    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent>;

    // Returns a channel receiver for display change notifications.
    // Windows: RegisterDeviceNotification(GUID_DEVINTERFACE_MONITOR)
    // X11: RandR RRScreenChangeNotify
    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent>;

    // Creates a transparent, click-through overlay window.
    // Must guarantee: mouse events pass through to windows underneath.
    // Windows: WS_EX_TRANSPARENT + WS_EX_LAYERED + WM_NCHITTEST → HTTRANSPARENT
    // X11: override_redirect + XShapeCombineRectangles with empty input shape
    fn create_overlay_window(&self, monitor_id: MonitorId) -> OverlayHandle;

    // Blits raw RGBA pixel buffer to the overlay window surface.
    // Pixel format: premultiplied BGRA (Windows), native-endian ARGB (X11)
    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32);

    fn destroy_overlay_window(&self, handle: OverlayHandle);
}
```

Monitor struct: `{ id, name, x, y, width, height, dpi_scale, is_primary }`
Window struct: `{ handle: WindowHandle, title, rect: Rect, is_visible }`
WindowMoveEvent: `{ handle: WindowHandle, event_type: DragStart | DragMove | DragEnd }`
DisplayChangeEvent: `{ event_type: Connected | Disconnected | ResolutionChanged }`

#### 2. MonitorManager

- Subscribes to `PlatformApi::subscribe_display_change_events()` (event-driven, not polling)
- Falls back to 30-second polling only as a safety net if native events are unavailable
- Computes monitor arrangement ID from EDID/serial + relative topology (fuzzy match, not exact pixels)
- Emits internal event when arrangement changes (hotplug detected)
- Provides `get_monitor_at(x, y)` for cursor-to-monitor lookup
- Pre-allocates overlay windows for max 4 monitors; reuses or creates on demand

#### 3. LayoutManager

- Owns all layout state in memory and on disk
- Layout = `HashMap<MonitorArrangementId, Vec<Zone>>`
- Zone = `{ id, name, x, y, width, height, gap, margin }` — stored as fractional coordinates (0.0–1.0 relative to monitor dimensions), converted to pixels at use time using `dpi_scale`
- **Gap:** space between adjacent zones within a monitor. A 10px gap means each zone's effective area is inset by 5px on each side. Only the zone interior (after gap) accepts snap targets.
- **Margin:** space between the zone's outer edge and the monitor boundary. A zone at top-left with margin=10 has its effective origin at (10,10).
- Per-zone gap/margin override the global Settings defaults.
- `activate_layout(id)`: switch active zones to match the given arrangement
- `save_layout(name, layout)`: persist a named layout to disk
- `list_layouts()`: return all saved layouts with their monitor arrangement IDs
- On hotplug: looks up arrangement ID → exact match preferred, fuzzy match as fallback → notifies user when fuzzy match is used → activates → falls back to default if no match
- Zones are non-overlapping within a monitor by construction. Creating a zone over an existing one clips or splits.

#### 4. DragDetector

- Consumes the `mpsc::Receiver<WindowMoveEvent>` from `PlatformApi` on a dedicated thread
- **Drag start** detected when:
  - `DragStart` event received AND `PlatformApi::is_mouse_button_down()` is true
  - Cursor is within the window's title bar region (platform heuristics)
  - Window has moved ≥ 5px from the first event (threshold filters out compositor noise)
- Before processing any drag event: check `AppState.is_paused`. If paused, discard the event.
- On drag start: records window handle and original size, tells ZoneOverlay to show on the **current monitor only** (not all monitors). Sets `drag_state.snap_in_progress = false`.
- During drag (per `DragMove` event):
  - Get cursor position, determine which monitor via `MonitorManager::get_monitor_at()`
  - When cursor crosses to a different monitor: show overlay on destination, hide on source
  - Look up zones for the current monitor, test point-in-zone
  - Tell ZoneOverlay to update highlight + ghost preview
- On `DragEnd`:
  - If cursor is inside a zone: set `drag_state.snap_in_progress = true`, call `PlatformApi::move_window(handle, zone_rect)`, then set `snap_in_progress = false` on next idle cycle
  - If cursor outside all zones: do nothing
  - ZoneOverlay hides
- **Self-trigger prevention:** While `snap_in_progress` is true, all `WindowMoveEvent` for the snapped window handle are discarded.

#### 5. ZoneOverlay

- Creates transparent overlay windows (one per active monitor) via `PlatformApi::create_overlay_window()`
- All overlay windows guarantee click-through (see PlatformApi section for platform specifics)
- Hidden by default, shown only on the monitor where a drag is active
- Renders: zone borders (2px, WCAG AA contrast accent color), highlighted zone fill (20% accent), ghost window preview (translucent filled rectangle showing snapped size)
- Uses `tiny-skia` to render into pre-allocated pixel buffers (buffer reused per frame, no per-frame allocation)
- Dirty-rect rendering: only repaint zones that changed since last frame (highlighted zone switched, ghost position moved)
- Calls `PlatformApi::overlay_present()` each frame to blit the pixel buffer to the overlay window
- On composited X11 desktops, overlay pixel updates may lag 1–2 frames behind cursor during very fast drags — this is inherent to the X11 rendering model

#### 6. TrayManager

- System tray icon (Tauri tray API)
- Menu: "Configure" (opens config window), "Pause/Resume" (toggles snapping), "View Logs" (opens log file), "Quit"
- Pause state: changes tray icon to indicate inactive status
- Toggling pause during an active drag immediately cancels the drag (hides overlay, clears drag_state, does NOT snap). The is_paused flag prevents new drag detection from the next event onward.
- On platforms without system tray (GNOME without extension): auto-open config window on startup and run headless with just the overlay — "Quit" available via config window

#### 7. ConfigStore

- Reads/writes `layouts.json` at:
  - Linux: `~/.config/grid-screen/layouts.json`
  - Windows: `%APPDATA%/GridScreen/layouts.json`
- Config file includes `schema_version` field for future format migrations
- Validation on load — rejects files that fail. Falls back to default layout on corruption. Does NOT overwrite corrupted file.
- Write strategy: rename current to `layouts.json.tmp`, write new, read-back verify, rotate `.tmp` into backup ring, delete `.tmp`
- Backup: up to 5 rotating backups named `layouts.json.bak.1` through `layouts.json.bak.5`
- Creates app data directory with `create_dir_all` if not present
- Auto-creates app data directory if missing

### Tauri IPC Configuration

**Capability permissions** (deny-by-default, explicit allowlist):
```json
{
  "identifier": "gridscreen:default",
  "windows": ["config-*"],
  "permissions": [
    "core:default",
    "tray:default",
    "core:window:allow-close",
    "core:window:allow-set-focus"
  ]
}
```

**Denied capabilities:** `shell:*`, `http:*`, `fs:*` (no filesystem access from webview; all config I/O through backend commands)

**IPC command catalog:**
```rust
#[tauri::command] fn get_current_state(state: State<AppState>) -> FrontendState;
#[tauri::command] fn apply_layout(state: State<AppState>, layout: Layout) -> Result<()>;
#[tauri::command] fn save_layout(state: State<AppState>, name: String, layout: Layout) -> Result<()>;
#[tauri::command] fn list_layouts(state: State<AppState>) -> Vec<SavedLayout>;
#[tauri::command] fn delete_layout(state: State<AppState>, id: LayoutId) -> Result<()>;
#[tauri::command] fn toggle_pause(state: State<AppState>) -> bool;
```

**Content Security Policy** (set in `tauri.conf.json`):
```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
connect-src 'self' ipc: https://ipc.localhost;
img-src 'self' data:;
```

**Tauri events** (backend → frontend state sync):
- `state-changed`: emitted when any state mutation occurs (layout switch, pause toggle, monitor change)
- `hotplug-detected`: emitted when monitor configuration changes, carries new monitor list

### Input Validation Rules

**Zone coordinates** (validated on load and on IPC apply):
- `x, y`: must be in [0.0, 1.0] (fractional), finite (no NaN/Infinity)
- `width, height`: must be > 0.0, ≤ 1.0
- Zone must be fully within monitor bounds: `x + width ≤ 1.0`, `y + height ≤ 1.0`
- Max 64 zones per monitor (capped on create, rejected on load)

**Layout names:**
- Max 64 characters, trimmed
- HTML special characters (`<`, `>`, `&`, `"`, `'`) are escaped before rendering in frontend (prevents stored XSS)
- Frontend must additionally use framework-level escaping (Svelte auto-escapes by default)

**Zone names:** Same rules as layout names.

**Zone overlap:** Rejected. Adjacent zones may touch (shared boundary at `zone1.x + zone1.width == zone2.x`) with a gap applied between.

### Frontend (Web) — 3 screens

#### Layout Editor

- Shows all monitors side-by-side, scaled to fit the config window
- Each monitor rendered as a rectangle with its aspect ratio preserved
- Configurable column grid (default 12, adjustable per-monitor 4–24)
- Zone operations:
  - **Create:** click-drag on empty monitor area → new zone snaps to nearest grid lines
  - **Resize:** drag handles on zone edges/corners → snap to grid lines during resize
  - **Move:** drag zone body to reposition
  - **Rename:** double-click zone label
  - **Delete:** right-click → "Delete zone"
- "Apply" button sends layout to backend immediately (live preview of new zones on desktop)
- "Save" persists it under a name

#### Layout Manager

- Lists all saved layouts with name and thumbnail (small rendered preview of zone layout)
- Rename, delete, duplicate layouts
- Assign layout to a specific monitor arrangement
- Set one layout as default

#### Settings

- Auto-start with system toggle (Linux: `.desktop` in `~/.config/autostart/`; Windows: `Run` registry key)
- Zone gap/margin defaults
- Accent color for overlays (with WCAG AA contrast preview)
- Language (Vietnamese / English)
- Check for updates (via Tauri updater), About, View Logs

### Accessibility

- Config UI supports keyboard navigation (Tab, Enter, Escape, arrow keys for zone movement)
- Zone editor canvas: zones are focusable, resizable via keyboard (arrow keys + Shift for fine-tuning)
- ARIA labels on all interactive elements in the config UI
- Overlay zone borders use WCAG AA contrast ratio against typical desktop backgrounds
- All UI text supports i18n framework (English + Vietnamese in v1)
- High-DPI support: zones stored in fractional coordinates, converted at render time using OS-reported DPI scale

### First-Run Experience

1. User installs and launches Grid Screen (or it auto-starts on login)
2. Tray icon appears. A tray notification is shown: "Grid Screen is ready. Click the tray icon to set up your zones."
3. If the user does nothing: a default 1-zone-per-monitor layout is active immediately — dragging windows already shows the zone highlight and snaps to full-monitor zones
4. User clicks tray → "Configure" → the config window opens to the Layout Editor
5. A subtle onboarding overlay in the editor: "Drag on a monitor to create your first zone" (shown once, dismissible)
6. After saving a layout: tray notification confirms "Layout saved. Drag any window into your zones!"
7. "Pause" available at any time from tray menu

### User-Facing Error States

| Situation | User sees |
|-----------|----------|
| Config file corrupted | Tray notification: "Layout reset to default — config file was damaged" |
| Overlay visual unavailable | Snapping still works (windows snap silently). Logged for diagnostics. |
| Wayland native limitation | Tray notification on first detection: "Some features limited on this display system. X11 apps work normally." |
| Monitor unplugged during drag | Drag cancels silently. No error visible. |
| Cannot save config | Tray notification: "Could not save layout. Trying again..." — retries automatically |
| No system tray available | Config window opens on startup. App runs headless. "Quit" available via window. |

## Data Flow

### Startup
```
App starts
  → Tray icon created
  → ConfigStore loads layouts from disk
  → MonitorManager subscribes to display change events + enumerates displays
  → Monitor arrangement ID computed
  → LayoutManager activates matching layout (fuzzy fallback if needed, or default)
  → DragDetector spawns thread, subscribes to window move event channel
  → First-run: show tray notification "Grid Screen is ready"
  → Ready
```

### Window drag-and-drop (event-driven)
```
Platform event-loop thread sends WindowMoveEvent through mpsc channel

DragDetector thread receives DragStart:
  → Check is_paused → discard if paused
  → Check is_mouse_button_down() → discard if false
  → Check window moved ≥ 5px → start tracking
  → Check snap_in_progress for this handle → discard if true
  → Record window handle + original size
  → ZoneOverlay::show(monitor_containing_cursor)

DragDetector receives DragMove:
  → Get cursor position
  → MonitorManager::get_monitor_at(cursor) → monitor
  → If monitor changed: move overlay to new monitor
  → LayoutManager::get_zones(monitor) → zones
  → Test point-in-zone for each zone → find containing zone
  → ZoneOverlay::update(highlighted_zone, ghost_rect) — dirty-rect only

DragDetector receives DragEnd:
  → If cursor in a zone:
     → Set snap_in_progress = true
     → PlatformApi::move_window(handle, zone_pixel_rect)
     → Queue snap_in_progress = false for next idle cycle
  → If cursor outside all zones: do nothing
  → ZoneOverlay::hide()
```

### Monitor hotplug (event-driven)
```
PlatformApi sends DisplayChangeEvent through channel
  → MonitorManager re-enumerates displays
  → Compute new arrangement ID
  → LayoutManager::activate_layout(new_id)
     → Exact match: apply zones (convert fractional coords to pixels using current dpi_scale)
     → Fuzzy match: apply zones + notify user "Layout matched approximately"
     → No match: show tray notification "New display setup detected. Open config to create a layout?"
```

### Config window open
```
User clicks "Configure" in tray
  → Tauri opens secondary webview window (config-ui)
  → Frontend calls IPC get_current_state()
     ← Returns: monitors, active layout, all saved layouts
  → User edits layout in editor
  → "Apply" → IPC apply_layout(layout) → backend validates, updates memory, emits state-changed
  → "Save"  → IPC save_layout(name, layout) → ConfigStore writes JSON with write-verify-backup
  → Close window: backend continues running, no state lost
```

## Threading Model

```
┌─────────────────────────────────────────────────────────┐
│                    THREAD TOPOLOGY                       │
│                                                          │
│  Main Thread (Tauri event loop)                          │
│    - TrayManager (icon + menu)                           │
│    - Tauri IPC command handlers (brief lock acquires)    │
│    - Webview rendering                                   │
│                                                          │
│  Platform Event Thread (spawned by PlatformApi impl)     │
│    - Runs OS message pump / event loop                   │
│    - Sends WindowMoveEvent → mpsc channel                │
│    - Sends DisplayChangeEvent → mpsc channel             │
│    - NEVER acquires application locks                    │
│                                                          │
│  Drag Processor Thread (spawned by DragDetector)         │
│    - Receives from mpsc channels                         │
│    - Acquires read locks on active_layout, monitors      │
│    - Computes zone hit-testing                           │
│    - Sends overlay update commands (lock-free channel)   │
│                                                          │
│  Overlay Render Thread (spawned by ZoneOverlay)           │
│    - Receives overlay update commands via channel        │
│    - Renders with tiny-skia (no app locks held)          │
│    - Calls PlatformApi::overlay_present()                │
│                                                          │
│  Monitor Polling Thread (safety net, 30s interval)       │
│    - Only if native display events unavailable           │
│    - Triggers re-enumeration on change                   │
└─────────────────────────────────────────────────────────┘
```

State access pattern:

```rust
// Lock-free reads for hotpath data (updated atomically on layout/monitor changes)
active_layout: Arc<ArcSwap<Layout>>,
monitors: Arc<ArcSwap<Vec<Monitor>>>,

// Contended only during drag start/end transitions
drag_state: Mutex<Option<DragState>>,

// Acquired briefly by config IPC, tray, monitor polling
app_config: RwLock<AppConfig>,  // is_paused, settings, saved_layouts list
```

- DragDetector reads `active_layout` and `monitors` lock-free via `ArcSwap::load()`
- ZoneOverlay receives update commands via its own `mpsc` channel (lock-free)
- Config IPC acquires `RwLock` briefly for reads, longer only for writes
- Overlay rendering runs entirely outside any lock

## Platform-Specific Details

### Windows

| Operation | Win32 API |
|-----------|-----------|
| Enumerate monitors | `EnumDisplayMonitors` + `GetMonitorInfoW` |
| Display change events | `RegisterDeviceNotification(GUID_DEVINTERFACE_MONITOR)` |
| Cursor position | `GetCursorPos` |
| Mouse button state | `GetAsyncKeyState(VK_LBUTTON)` |
| Move/resize window | `SetWindowPos` with `HWND_TOP` |
| Enumerate windows | `EnumWindows` |
| Detect window drag | `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)` on dedicated message-pump thread |
| Overlay window | `CreateWindowExW` with `WS_EX_LAYERED \| WS_EX_TRANSPARENT \| WS_EX_TOOLWINDOW \| WS_EX_NOACTIVATE` |
| Overlay click-through | `WM_NCHITTEST` → return `HTTRANSPARENT` for entire window |
| Overlay draw | `UpdateLayeredWindow` with `BLENDFUNCTION`, pixel buffer → DIB via `BITMAPINFO` |
| Auto-start | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registry key |

Crate: `windows` (official Microsoft Rust crate).

### Linux

#### X11

Crate: `x11rb` (pure Rust, async-safe).

| Operation | X11 API |
|-----------|---------|
| Enumerate monitors | `xrandr` extension or `Xinerama` |
| Display change events | `XRRSelectInput` with `RRScreenChangeNotifyMask` |
| Cursor position | `XQueryPointer` |
| Mouse button state | `XQueryPointer` (button mask in returned state) |
| Move/resize window | `XMoveResizeWindow` |
| Detect window drag | `ConfigureNotify` via `SubstructureRedirectMask` on root window, dedicated event-loop thread |
| Overlay window | `CreateWindow` with `override_redirect = True`, transparent background pixel |
| Overlay click-through | `XShapeCombineRectangles` with empty input shape bounding rectangle |
| Overlay draw | `XCreatePixmap` from raw pixel buffer, `XCopyArea` to overlay window |
| Auto-start | `.desktop` file in `~/.config/autostart/` |

**X11 security note:** X11 has no isolation between clients running as the same user. Any X11 application can intercept or spoof window events. This is inherent to X11's architecture and cannot be fully mitigated. Wayland native support (Phase 2) resolves this at the compositor level. Overlay window z-order is verified periodically as a defense-in-depth measure.

#### Wayland

Wayland restricts third-party apps from manipulating other windows. Strategy:

**Wayland session detection:** Attempt connection to `$WAYLAND_DISPLAY` socket. If Wayland is present, attempt X11 connection through `$DISPLAY`. If both available → XWayland mode. If only Wayland → notify user of limitation.

- **Phase 1:** Detect Wayland, fall back to XWayland windows only. Show notification: "Some features limited on this display system. X11 apps work normally."
- **Phase 2:** Use `ext-foreign-toplevel-list` protocol (supported by KDE Plasma 5.27+, wlroots compositors). GNOME/Mutter requires a separate extension. May require explicit user permission in compositor settings.
- Crates: `wayland-client`, `wayland-protocols-wlr`

XWayland apps are treated as X11 windows and work fully on Phase 1.

## Error Handling & Logging

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
enum ConfigError { Io, Parse, Validation, SchemaMismatch }
enum PlatformError { WindowNotFound, PermissionDenied, Unsupported, Internal }
enum LayoutError { ZoneOverlap, ZoneOutOfBounds, TooManyZones, InvalidName }
```

### Graceful Degradation

Guiding principle: the background app must never crash. All errors degrade gracefully.

| Scenario | Handling |
|----------|----------|
| Corrupted JSON config | Log ERROR, fall back to default layout, do NOT overwrite corrupted file |
| Overlay window creation fails | Log WARN, disable overlay visuals, snapping still works |
| Wayland restricting window moves | Log INFO, show tray notification, disable snap for native Wayland windows |
| Monitor disconnected during drag | DragDetector cancels drag state, ZoneOverlay hides, no crash; log DEBUG |
| Target window closed before snap | `move_window` returns `WindowNotFound`, skip silently, log DEBUG |
| Display event transient error | Retry 3 times, keep last known monitor list if all fail; log WARN |
| Cannot write config file | Log ERROR, keep data in memory, retry after 5 minutes; tray notification to user |

### Logging

- Framework: `tracing` crate with `tracing-subscriber` and `tracing-appender` for file rotation
- Log file: `~/.config/grid-screen/grid-screen.log` (Linux) / `%APPDATA%/GridScreen/grid-screen.log` (Windows)
- Rotation: 3 files, 1 MB each (non-blocking writer)
- Log levels: ERROR (unrecoverable), WARN (degraded), INFO (state transitions, layout switch, monitor change), DEBUG (per-drag-event cursor/zone data, frame timing)
- "View Logs" menu item in tray opens the log file in the default text editor

## Performance Budget

| Metric | Target |
|--------|--------|
| Drag overlay FPS | ≥ 60 FPS |
| Drag latency (cursor move → overlay update) | ≤ 1 frame (16ms) |
| Snap latency (mouse release → window in final position) | ≤ 200ms |
| CPU usage during active drag | < 15% of one core (4-monitor setup, 16 zones per monitor) |
| Idle CPU usage | < 0.5% |
| Startup time (launch → tray icon visible) | < 500ms |
| Memory (idle) | < 60 MB |
| Memory (during drag, 4K monitor) | < 100 MB (pre-allocated buffers) |
| Config save latency | < 50ms |

### Instrumentation

- `tracing` spans on the drag loop for per-frame timing
- FPS counter rendered in overlay corner in dev builds (toggle via config)
- Smoke test in CI: simulated drag of 30 seconds with 64 zones, verify FPS ≥ 60

## Testing Strategy

| Type | Scope |
|------|-------|
| Rust unit tests | Zone geometry math (point-in-zone, overlap detection), configuration ID fuzzy matching, JSON serialization/deserialization with validation, DPI coordinate conversion, gap/margin layout math |
| Rust integration tests | Mock `PlatformApi` to test DragDetector event processing (drag start/end transitions, snap_in_progress filtering, pause during drag, monitor boundary crossing), MonitorManager arrangement change handling, LayoutManager layout matching — no real display needed |
| Rust benchmarks | Frame time measurement: zone hit-testing for 64 zones, overlay rendering with tiny-skia |
| Security | `cargo audit` and `cargo deny` in CI to catch dependency vulnerabilities |
| Memory | `valgrind` / `dhat` pass at milestone 1 to verify no allocation leaks |
| Manual QA | Visual overlay rendering, drag-and-drop UX feel, multi-monitor hotplug, system tray behavior, first-run experience |
| No automated E2E | OS-level interaction is too fragile to automate; manual testing is sufficient for initial release |

## Distribution & CI/CD

### Packaging

- **Windows:** NSIS or MSI installer built via Tauri bundler, signed with code-signing certificate (SmartScreen compatibility)
- **Linux:** AppImage (primary, portable) and `.deb` (Debian/Ubuntu) via Tauri bundler
- **Auto-updates:** Tauri updater plugin, checking GitHub Releases for version manifest
- **Versioning:** Semver (`MAJOR.MINOR.PATCH`)

### CI/CD (GitHub Actions)

Matrix build: `ubuntu-latest` (Linux X11 target) + `windows-latest`

Pipeline per push/PR:
1. `cargo fmt --check` + `cargo clippy -- -D warnings`
2. `cargo test` (unit + integration)
3. `cargo audit` + `cargo deny check`
4. Build release artifacts (`cargo tauri build`)
5. Publish artifacts to GitHub Releases on tag push

### Development Environment

Prerequisites:
- Rust toolchain (stable, via `rustup`)
- Node.js 20+ (for Svelte frontend build)
- Linux: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libx11-dev`, `libxrandr-dev`, `libxinerama-dev`
- Windows: Visual Studio Build Tools with C++ workload, Windows 10 SDK

Run in dev mode: `cargo tauri dev`

## Constraints and Known Limitations

- Wayland native window snapping will not work in Phase 1 (XWayland apps will work)
- Default 1-zone-per-monitor layout is auto-created on first launch — snapping works immediately
- On composited X11 desktops, overlay pixel updates may lag 1–2 frames behind cursor during very fast drags (inherent X11 rendering model limitation)
- App requires system tray support; on DEs without tray (e.g., stock GNOME), config window opens on startup and app runs headless
- X11 has no security boundaries between clients — any X11 app can observe or interfere with window events (mitigated in Wayland Phase 2)
- Max 64 zones per monitor (enforced at create and load time)
- Layout names and zone names capped at 64 characters

## Not In Scope (v1)

- Keyboard shortcuts for snap operations (drag-only in v1)
- Per-application zone assignment rules (e.g., "always put Chrome in zone 2")
- Touchscreen / tablet support
- macOS support
- Wayland native full support (Phase 2)
- Window grouping (snap multiple windows together)
- Cloud sync for layouts across machines
