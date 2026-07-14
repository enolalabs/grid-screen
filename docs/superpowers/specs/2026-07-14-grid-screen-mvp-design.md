# Grid Screen MVP Design

**Date:** 2026-07-14  
**Status:** Approved design  
**UI reference:** `mockups/design-1-aurora-dark.html`  
**Product language:** English  
**Initial platform:** Linux X11

This spec replaces `2026-07-11-grid-screen-greenfield-design.md`. The mockup is the source of truth; the spec is derived from it.

## 1. Summary

Grid Screen is a Linux X11 desktop application that lets a user arrange application windows on a selected screen by assigning them to zones in a predefined layout and applying the arrangement as a batch.

The user opens the configuration window, chooses a screen and a layout preset, drags window cards into zones on a canvas preview, and clicks **Arrange**. The application then moves and resizes each assigned window to its target zone.

The application runs with a system tray icon. Closing the configuration window keeps the process running in the tray.

## 2. Product Goals

### 2.1 Goals

- Let a user arrange three windows in under 60 seconds.
- Make the target position of every selected window explicit on a canvas preview before applying.
- Keep layouts simple (2-3 zone grids, no freeform editing).
- Remain lightweight and stable while running in the background.
- Fail visibly when X11 or the window manager rejects an operation.

### 2.2 Non-goals for the MVP

- Wayland, Windows, or macOS implementations.
- Modifier-assisted snap (drag a real system window with a modifier key held).
- A visual layout editor with draggable dividers. Layouts are adjusted via sliders only.
- Onboarding flow. The app opens directly to the Arrange view.
- Keyboard accessibility (focus management, keyboard alternatives for drag-and-drop).
- Moving windows between Linux virtual desktops/workspaces.
- Launching applications automatically.
- Persisting application-to-zone mappings (assignments are ephemeral).
- Freeform or overlapping zones.
- Rollback on partial arrangement failure.
- Cloud sync, user accounts, remote content, analytics, or telemetry.
- Localization beyond English.

## 3. Target User and Principles

The primary user is a general desktop user who wants to divide a screen without manually resizing every window. The UI must not require knowledge of X11, EWMH, window handles, or coordinates.

Principles:

- Presets first: useful layouts are available immediately.
- Direct manipulation: the user drags window cards onto zones.
- Safe by default: unassigned windows are never changed.
- Explicit activation: batch movement occurs only after the user clicks **Arrange**.
- Graceful degradation: unsupported capabilities are disabled with an explanation.

## 4. Supported Environment

### 4.1 MVP platform contract

- Linux desktop session using X11.
- The current Linux virtual desktop/workspace only.
- One target screen at a time, selected by the user.
- Running windows may originate from any connected screen, provided they belong to the current workspace.
- Screen geometry comes from XRandR.
- Window metadata and window-manager requests use EWMH where available.

If the process detects Wayland without X11, it displays an "X11 required" notice and disables the arrange action.

### 4.2 Eligible windows

The Window Catalog includes normal top-level application windows in the current workspace, including minimized windows and windows on other screens. It excludes:

- Grid Screen's own windows.
- Desktop, dock, panel, notification, menu, tooltip, and dialog-only popup windows.
- Fullscreen windows.
- Windows that the window manager reports as not movable or not resizable.
- Windows that disappear before catalog validation completes.

Multiple windows from one application are listed separately using application name and window title. If a title is empty, the application name and a stable per-session ordinal are used.

Window IDs are opaque, session-only values. They are never stored in a layout or config file.

## 5. User Experience

### 5.1 Application lifecycle

Grid Screen opens directly to the **Arrange** view. There is no onboarding.

Closing the configuration window minimizes the app to the system tray. The tray icon provides:

- **Open Grid Screen** — opens the configuration window.
- **Quit** — exits the application.

If no system tray is available, closing the window quits the application.

An "Start at Login" toggle in Settings controls whether the app auto-launches. It is off by default.

A "Minimize to Tray" toggle in Settings controls whether closing the window quits or minimizes to tray. It is on by default.

### 5.2 Navigation

The configuration window has three top-level tabs:

- **Arrange** — select a target screen and layout, assign windows to zones, apply the arrangement.
- **Layouts** — browse presets, duplicate, create, and delete saved layouts.
- **Settings** — snap behavior, modifier key, autostart, margin/gap defaults, system status.

**Arrange** is the default tab.

### 5.3 Arrange View

Three-column layout (280px | 1fr | 300px):

**Left: Window Catalog**

- Search box to filter windows by app name or title.
- Scrollable list of window cards, each showing: app icon (color-coded), app name, window title.
- Cards are draggable. An assigned card is visually dimmed, non-draggable, and shows a **Zone N** badge.
- Empty state: "No windows found" when search yields no results.

**Center: Canvas**

- **Toolbar**: screen selector dropdown (auto-selects the screen containing the Grid Screen window), layout selector dropdown, quick layout icon buttons.
- **Canvas preview**: a visual representation of the selected screen showing zones as a CSS grid. Assigned zones show the window's app name and a remove (X) button. Empty zones show "Drop window here".
- **Action bar**: "Clear All" secondary button and "Arrange N windows" primary button (disabled until at least one window is assigned). Clicking arrange validates assignments, then moves/resizes each window. On success, a toast appears and assignments are cleared. On failure, a toast describes what went wrong.

**Right: Detail Panel**

- Divider Ratio slider (10%–90%, only active for 2-zone layouts).
- Gap slider (0–40 px, space between zones).
- Margin slider (0–60 px, space around screen edge).
- Snap section: Enable Snap toggle, Show Overlay Zones toggle. Present in the UI per the mockup but non-functional in the MVP (modifier-assisted snap is deferred).
- System Status: session type, EWMH support, workspace, active screen.

### 5.4 Layouts View

Grid of layout cards:

- **Built-in presets** (5 cards): Two Columns, Three Columns, Focus + Stack, Main + Sidebar, 3 Wide Center. Each has a mini grid preview, name, and "Use" / "Duplicate" actions.
- **Saved layouts**: user-created layouts, shown below presets with "Use" / "Edit" / "Delete" actions.
- **"+ New Layout"** button opens a simple creation flow: choose a base preset, adjust sliders, name the layout, save.
- Clicking a layout card selects it and navigates to Arrange.

Layouts are adjusted via sliders (Ratio, Gap, Margin) in the Arrange detail panel. There is no standalone visual layout editor with draggable dividers.

### 5.5 Settings View

Three setting groups plus System Status:

- **Snap Behavior**: Enable Modifier Snap toggle, Snap Modifier Key dropdown (Shift / Ctrl / Alt / Super). Non-functional placeholder UI in the MVP; modifier-assisted snap is deferred.
- **Defaults**: Default Gap (read-only, derived from last used) and Default Margin (read-only, derived from last used).
- **General**: Start at Login toggle (off by default), Minimize to Tray toggle (on by default).
- **System Status**: Session Type, EWMH Support, Window Manager name, XRandR availability, Current Workspace, Connected Screens. Errors and capability degradations are displayed here when applicable.

## 6. Layout Model

### 6.1 Structure

A layout is a flat grid definition with 2 or 3 zones:

```
Layout {
  id: string
  name: string
  type: "preset" | "saved"
  zones: 2 | 3
  columns: string           // CSS grid-template-columns (e.g. "1fr 1fr", "2fr 1fr")
  rows?: string             // CSS grid-template-rows (only for Focus+Stack: "1fr 1fr")
  span_first?: boolean      // zone 1 spans 2 rows (Focus+Stack only)
  ratio?: number            // 10-90, for 2-zone layouts, determines column split
  gap_px: number            // 0-40
  margin_px: number         // 0-60
  created_at: string
  updated_at: string
}
```

Ratios are integers in the range 10-90 representing the percentage of width given to the first column.

### 6.2 Built-in presets

| Preset | Zones | Columns | Rows | span_first | ratio |
|---|---|---|---|---|---|
| Two Columns | 2 | `1fr 1fr` | — | — | 50 |
| Three Columns | 3 | `1fr 1fr 1fr` | — | — | — |
| Focus + Stack | 3 | `2fr 1fr` | `1fr 1fr` | true | — |
| Main + Sidebar | 2 | `3fr 1fr` | — | — | 75 |
| 3 Wide Center | 3 | `1fr 2fr 1fr` | — | — | — |

Adding a new layout from the Layouts view starts from a preset base, then the user adjusts sliders in Arrange and saves.

### 6.3 Geometry derivation

1. Obtain target screen work area (XRandR rectangle minus EWMH reserved struts).
2. Subtract margin from all four edges.
3. Divide the remaining rectangle according to column/row fractions and ratio.
4. Subtract half of gap from each shared edge.
5. Round pixel coordinates using deterministic floor/ceil to fill the work area without gaps or overlaps.

Every final zone must have a positive width and height.

## 7. Technical Architecture

### 7.1 Technology choices

| Layer | Choice |
|---|---|
| Desktop shell | Tauri 2 |
| Application core | Rust |
| Configuration UI | Svelte with TypeScript |
| MVP platform integration | Linux X11 adapter |
| Persistence | Versioned JSON in `~/.config/grid-screen/` |
| UI/core communication | Typed Tauri commands and events |

### 7.2 Process model

```
Svelte configuration UI (webview)
        ↕ typed commands and state events
Rust application core
        ↕ PlatformAdapter trait
Linux X11 implementation
```

The Svelte webview may be created and destroyed without stopping the Rust core.

### 7.3 Core components

**App Shell** — Owns Tauri lifecycle, configuration window, system tray, single-instance behavior, clean shutdown, autostart integration.

**Window Catalog** — Enumerates X11 windows in the current workspace, applies eligibility rules, creates display-safe descriptors. Exposes no native handles outside Rust.

**Layout Engine** — Derives pixel zone rectangles from a Layout definition and screen work area. Enforces gap, margin, and minimum-size constraints.

**Arrange Orchestrator** — Validates assignments, restores minimized windows, moves and resizes each assigned window sequentially, reports per-window results.

**Config Store** — Loads, validates, migrates, and atomically writes settings and layouts. Uses temp-file-write + rename pattern. Keeps a small rotating backup set.

**PlatformAdapter** (trait) — Operating-system boundary:

```
enumerate_screens() -> Vec<ScreenInfo>
current_workspace() -> WorkspaceId
enumerate_windows(workspace) -> Vec<WindowDescriptor>
get_window_state(window_id) -> WindowState
restore_window(window_id) -> ()
move_resize_window(window_id, rect) -> Result<Rect, Error>
subscribe_workspace_events() -> EventStream
subscribe_screen_events() -> EventStream
```

Screen IDs and window IDs are opaque. The adapter returns capability information so the core can disable unsupported features.

### 7.4 Linux X11 adapter

Uses XRandR for screens and geometry, EWMH properties for window metadata and workspace, and X11/XInput for workspace and display change events. All X11 resources are released on shutdown.

## 8. Data Flows

### 8.1 Batch arrangement

1. Svelte requests screens, layouts, and window catalog from Rust.
2. User drags window cards into zones (pure Svelte state).
3. User clicks **Arrange N windows**.
4. Svelte sends `arrange_windows` command with layout ID, screen ID, and assignment map.
5. Arrange Orchestrator refreshes catalog and validates: every window still exists and is movable/resizable.
6. If validation fails: return structured errors, no windows changed, toast in UI.
7. Layout Engine computes zone rectangles for the current screen work area.
8. Minimized windows are restored, then each window is moved and resized.
9. Per-window results returned to Svelte.
10. Svelte shows success toast and clears assignments, or shows error toast and keeps assignments.

Arrangement commands are serialized so two cannot race.

## 9. Failure Handling

| Condition | Behavior |
|---|---|
| Wayland or missing X11 control | Show "X11 required" notice, disable arrange. |
| Assigned window closed before arrange | Reject batch, mark stale card, show toast with count. |
| Window manager rejects movement | Report affected window, show toast. No rollback. |
| Application enforces size constraints | Keep the bounds the WM accepted, report the limitation if notable. |
| Target screen disconnects | Cancel, clear assignments, prompt new screen selection. |
| Workspace changes | Clear assignments, refresh catalog. |
| Config parse or validation failure | Preserve invalid file, load newest valid backup, fall back to defaults. |

Transient errors use toasts. Persistent environment problems appear in Settings > System Status.

## 10. Persistence and Security

Configuration stored under `${XDG_CONFIG_HOME:-~/.config}/grid-screen/` as versioned JSON.

```
Settings {
  schema_version: u32
  snap_enabled: bool
  snap_modifier: string
  autostart_enabled: bool
  minimize_to_tray: bool
  last_layout_id: Option<string>
  active_target_screen_hint: Option<string>
  default_gap_px: u32
  default_margin_px: u32
}
```

Layouts are stored as an array alongside settings.

Writes use temp file, flush, validation read-back, and atomic rename. A small rotating backup set is kept (last 5 files).

Only the Rust core accesses the filesystem and native window APIs. The webview receives a narrow command allowlist. The content security policy permits local packaged assets and Tauri IPC only. User-provided layout names and platform-provided application titles are rendered as text, never injected as HTML.

## 11. Accessibility

MVP targets mouse interaction only. Accessibility requirements (keyboard navigation, ARIA, focus management) are deferred to a future iteration.

## 12. Performance Targets

- Idle CPU below 1% on a reference Linux desktop.
- Batch arrangement of three supported windows completes within one second.
- No continuous cursor or geometry polling while idle.
- An eight-hour idle soak shows no unbounded memory, X11 resource, or thread growth.

## 13. Testing Strategy

### 13.1 Unit tests

- Layout Engine: pixel derivation, ratio clamping, gap/margin enforcement, minimum-size constraints, rounding determinism.
- Config Store: serialization/deserialization, schema migration, atomic write, backup rotation, corruption recovery.
- Window Catalog: eligibility filtering rules.
- Arrange Orchestrator: validation rules, serialized execution.

### 13.2 Integration tests

A `MockPlatformAdapter` covers:

- Successful batch arrangement.
- Minimized-window restoration.
- Windows on a different screen.
- Validation failure with zero mutations.
- Mid-batch platform error.
- Screen disconnect and workspace change.
- Adapter capability degradation.

### 13.3 UI tests

- Screen and layout selection.
- Window-card drag assignment and reassignment.
- Remove assignment via X button.
- Clear All.
- Arrange success and error toasts.
- Tab navigation (Arrange, Layouts, Settings).
- Layout creation from preset.
- Settings toggles and modifier key selection.
- System Status display.
- Tray minimize/restore.

### 13.4 Manual compatibility

Tested on GNOME Xorg, KDE Plasma X11, and Xfce with mixed resolutions, minimized windows, and common applications.

## 14. Acceptance Criteria

1. A user can choose a preset, assign three running windows, and arrange them in under 60 seconds without instruction.
2. Each assigned window lands within the target zone (accounting for window decoration differences).
3. No test operation controls a window outside the current workspace or a window other than the one selected.
4. Batch arrangement completes within one second on the reference environment.
5. Idle CPU <1% with no continuous polling.
6. Closing windows, disconnecting a screen, changing workspace, or corrupting config does not crash the process.
7. Closing and reopening the configuration window works correctly while the tray process is alive.
8. Wayland and missing X11 capabilities produce an explicit unsupported state, not silent malfunction.
