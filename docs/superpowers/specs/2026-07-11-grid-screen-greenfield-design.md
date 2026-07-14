# Grid Screen Greenfield MVP Design

**Date:** 2026-07-11  
**Status:** Approved design  
**Product language:** English  
**Initial platform:** Linux X11

## 1. Summary

Grid Screen is a cross-platform desktop window-arrangement application. The first release targets Linux X11 and lets a user arrange application windows on one selected screen in two ways:

1. Choose a layout, assign running windows to zones, and arrange them as a batch.
2. Hold a configurable modifier key while dragging a window and drop it into a visible zone.

The application is designed for general, non-technical users. It starts with a short onboarding flow, then runs in the system tray so snapping remains available while the configuration window is closed.

This is a greenfield design. The previous implementation in Git history may be consulted for X11 failure modes, but its code and architecture are not the implementation baseline.

## 2. Product Goals

### 2.1 Goals

- Let a first-time user arrange three windows in less than 60 seconds.
- Make the target position of every selected window explicit before applying a layout.
- Support both batch arrangement and modifier-assisted drag snapping.
- Keep layouts independent of screen resolution.
- Remain lightweight and stable while running in the background.
- Isolate operating-system integration so future platforms do not leak into product logic.
- Fail visibly and safely when X11, the window manager, or an application rejects an operation.

### 2.2 Non-goals for the MVP

- Wayland, Windows, or macOS implementations.
- Simultaneous arrangement across multiple screens.
- Moving windows between Linux virtual desktops/workspaces.
- Launching applications automatically.
- Persisting application-to-zone mappings.
- Freeform or overlapping zones.
- Replacing a tiling window manager.
- Cloud sync, user accounts, remote content, analytics, or telemetry.
- Localization beyond English.

## 3. Target User and Principles

The primary user is a general desktop user who wants to divide a screen without manually resizing and aligning every window. The UI must not require knowledge of X11, EWMH, window handles, coordinates, or tiling terminology.

The product follows these principles:

- Presets first: a useful layout is available immediately.
- Direct manipulation: the user sees zones and drags window cards onto them.
- Safe by default: unassigned windows are never changed.
- Explicit activation: batch movement occurs only after the user clicks the arrange action.
- Quiet background behavior: drag snapping requires a modifier by default.
- Graceful degradation: unsupported capabilities are disabled with an explanation.

## 4. Supported Environment

### 4.1 MVP platform contract

- Linux desktop session using X11.
- The current Linux virtual desktop/workspace only.
- One target screen at a time, selected by the user.
- Running windows may originate from any connected screen, provided they belong to the current workspace.
- Screen geometry comes from XRandR.
- Window metadata and window-manager requests use EWMH where available.

If the process detects Wayland without an X11 session that provides the required control surface, it opens an `X11 required` explanation and disables arrangement and snapping. It must not imply that partial Wayland support is reliable.

### 4.2 Eligible windows

The Window Catalog includes normal top-level application windows in the current workspace, including minimized windows and windows on other screens. It excludes:

- Grid Screen's own windows and overlays.
- Desktop, dock, panel, notification, menu, tooltip, dialog-only popup, and other system windows.
- Fullscreen windows.
- Windows that the window manager reports as not movable or not resizable.
- Windows that disappear before catalog validation completes.

Multiple windows from one application are listed separately using application name plus window title. If a title is empty, the application name and a stable per-session ordinal are used.

Window IDs are opaque, session-only values. They are never stored in a layout or config file.

## 5. User Experience

### 5.1 Application lifecycle

On first launch, Grid Screen opens a three-step onboarding flow:

1. Explain batch arrangement.
2. Explain modifier-assisted drag snapping.
3. Ask whether Grid Screen may start automatically at login.

Autostart is opt-in. Completing onboarding opens the `Arrange` screen. Closing the configuration window keeps the core and tray process running.

On subsequent login launches, Grid Screen starts in the tray without opening the configuration window. The user opens the window from the tray when needed. If no tray is available, Grid Screen opens and keeps the configuration window accessible and provides an in-window Quit action.

The tray menu contains:

- Open Grid Screen
- Enable Snap / Disable Snap
- Quit

### 5.2 Navigation

The configuration window has three top-level destinations:

- **Arrange:** select a target screen and layout, then assign windows to zones.
- **Layouts:** create, edit, rename, duplicate, and delete layouts.
- **Settings:** snap behavior, modifier key, autostart, margin, and gap defaults.

`Arrange` is the default destination.

The Settings screen contains an expandable `System Status` section for session type, X11/EWMH capabilities, active workspace, target screen, and recoverable environment problems. It is not a fourth top-level destination.

### 5.3 Batch-arrangement flow

1. The app selects the screen containing the Grid Screen configuration window by default.
2. The user may choose another connected screen.
3. The user selects a built-in preset or saved layout.
4. The left panel lists eligible running windows from the current workspace.
5. The user drags a window card into a zone on the screen canvas.
6. A window can occupy at most one zone, and a zone can contain at most one window.
7. A card may be removed from a zone or dragged to another zone.
8. The primary action states the number of assigned windows, for example `Arrange 3 windows`.
9. Clicking the action validates all assignments, then performs the arrangement.
10. The UI reports success or identifies assignments that need attention.

Zones may remain empty. Empty zones and unassigned windows are unchanged.

Selecting a layout and target screen in Arrange immediately makes that pair the active snap target and persists the best-effort selection hints. It does not move any window. Only `Arrange windows` or a valid modifier-assisted drop moves a window. Editing the active layout updates the snap geometry only after the user saves the edit.

Drag-and-drop is not the only assignment mechanism. A focused window card exposes `Assign to zone`, and each occupied zone exposes `Remove assignment`, so the workflow is keyboard accessible.

### 5.4 Layout editor flow

The editor starts from a small preset gallery, including:

- Two equal columns.
- Three equal columns.
- Focus plus vertical stack.
- Main plus sidebar.
- Three columns with a wider center.

After choosing a preset, the user customizes it by dragging horizontal or vertical dividers. The editor always partitions the complete screen work area. It does not allow floating, overlapping, or disconnected zones.

The user can adjust:

- Divider positions.
- Gap between adjacent zones.
- Margin around the outer screen edge.
- Layout name.

The preview shows the usable screen area rather than the full physical screen, so panels and reserved desktop areas are not presented as available space.

### 5.5 Modifier-assisted snap flow

The default modifier is `Shift`, and the user may change it in Settings.

1. Snap is enabled and an active layout and target screen exist.
2. The user holds the configured modifier while dragging an eligible top-level window.
3. When the pointer enters the active target screen, a click-through overlay shows its zones.
4. The zone under the pointer is highlighted.
5. Releasing the mouse inside a zone moves and resizes the window to that zone.
6. Releasing outside every zone performs no arrangement.

The operation is cancelled and the overlay is hidden if the modifier is released, the workspace changes, snap is disabled, the target screen disconnects, the window disappears, or the drag becomes invalid.

## 6. Layout Model

### 6.1 Partition tree

A layout is a binary partition tree rather than a collection of arbitrary rectangles:

```text
Node = Zone(zone_id)
     | Split(axis, ratio_basis_points, first, second)

axis = Horizontal | Vertical
ratio_basis_points = integer from 1 through 9,999
```

`ratio_basis_points` defines the share given to the first child out of 10,000. Integer fixed-point ratios make serialization and geometry deterministic.

The editor constrains divider movement so every visible leaf remains at least 10% of the preview dimension along the split axis and at least 120 by 80 logical pixels on the current preview screen. If a saved layout is applied to a smaller screen, the runtime preserves ratios and clamps final pixel rectangles to non-negative bounds; it never changes the persisted tree implicitly.

### 6.2 Geometry derivation

The Layout Engine derives zone rectangles in this order:

1. Obtain the target screen's work area.
2. Apply the configured outer margin.
3. Recursively divide the remaining rectangle using the partition tree.
4. Apply half of the configured gap to each shared zone edge.
5. Round using a deterministic remainder rule so the un-gapped partition covers the complete inner work area without missing or overlapping pixels.

Margin and gap are non-negative logical pixels. The UI caps both values so every final zone retains a positive usable rectangle.

The X11 adapter computes per-screen work areas from XRandR screen rectangles and EWMH reserved areas/struts. If the window manager does not expose per-screen work areas, it intersects the available global work area with each screen rectangle. If neither source is reliable, it uses the screen rectangle and reports reduced capability in `Settings > System Status`.

### 6.3 Persisted entities

```text
Layout {
  id
  name
  partition_tree
  gap_logical_px
  margin_logical_px
  created_at
  updated_at
}

Settings {
  schema_version
  onboarding_completed
  snap_enabled
  snap_modifier
  autostart_enabled
  last_layout_id
  active_target_screen_hint
  default_gap_logical_px
  default_margin_logical_px
}
```

`active_target_screen_hint` is a best-effort monitor identity used only to restore the last selection. A disconnected or ambiguous screen is never selected silently; the user is asked to choose.

Window assignments are ephemeral and are cleared when the selected layout or workspace changes. Changing the target screen preserves assignments only if every assigned window is still valid; otherwise stale assignments are removed with an explanation.

## 7. Technical Architecture

### 7.1 Technology choices

| Layer | Choice |
|---|---|
| Desktop shell | Tauri 2 |
| Application core | Rust |
| Configuration UI | Svelte with TypeScript |
| MVP platform integration | Linux X11 adapter |
| Persistence | Versioned JSON in the XDG config directory |
| UI/core communication | Typed Tauri commands and events |

Electron was rejected because its background resource cost is unnecessary while native window control still requires a separate native layer. A Qt-only implementation was rejected because the visual editor and UI iteration cost would be higher without removing the need for platform-specific window code.

### 7.2 Process model

Grid Screen is one desktop process with three boundaries:

```text
Svelte configuration UI
        ↕ typed commands and state events
Rust application core
        ↕ PlatformAdapter
Linux X11 implementation
```

The Svelte webview may be created and destroyed without stopping the Rust core. Real-time drag detection, hit testing, overlay updates, and window movement remain in Rust and do not cross the webview IPC boundary.

### 7.3 Core components

#### App Shell

Owns the Tauri lifecycle, configuration window, onboarding state, system tray, autostart integration, single-instance behavior, and clean shutdown.

#### Window Catalog

Enumerates platform windows, applies eligibility rules, creates display-safe descriptors, and refreshes stale runtime IDs. It exposes no native handles outside the Rust process.

#### Layout Engine

Validates partition trees, derives zone rectangles from screen work areas, updates divider ratios, and enforces gap, margin, and minimum-size invariants.

#### Arrange Orchestrator

Validates an entire assignment set, snapshots original window state, restores minimized windows, requests movement, verifies results, and performs best-effort rollback on partial failure.

#### Snap Controller

Consumes platform drag and modifier events as a state machine. It resolves the active zone, tells the overlay what to render, and requests exactly one move/resize after a valid drop.

#### Overlay Controller

Maintains a transparent, always-on-top, click-through overlay for the active target screen. It renders zone outlines and the current highlight using a reused pixel buffer. It is hidden when there is no valid snap interaction.

#### Config Store

Loads, validates, migrates, and atomically writes settings and layouts. It keeps a small rotating backup set and never overwrites an invalid source file before preserving it for diagnosis.

#### PlatformAdapter

Defines the operating-system boundary. Conceptually it provides:

```text
enumerate_screens()
current_workspace()
enumerate_windows(workspace)
get_window_state(window_id)
restore_window(window_id)
move_resize_window(window_id, rect)
subscribe_window_drag_events()
subscribe_modifier_events()
subscribe_screen_and_workspace_events()
create_click_through_overlay(screen_id)
present_overlay(overlay_id, frame)
destroy_overlay(overlay_id)
```

Screen IDs and window IDs are opaque. The adapter returns capability information so the core can disable unsupported behavior instead of guessing.

### 7.4 Linux X11 adapter

The MVP adapter uses:

- XRandR for connected screens and geometry.
- EWMH properties for current workspace, window types, states, allowed actions, active window, and move/resize requests.
- X11/XInput events for pointer, button, modifier, window configuration, workspace, and display changes where supported.
- A native override-redirect overlay with an empty input shape so pointer events pass through.

The adapter may use a low-frequency safety refresh only when the window manager does not emit a required catalog-change signal. It must not continuously poll cursor or window geometry while idle. Higher-frequency pointer tracking is permitted only during an active modifier-assisted drag.

All X11 resources and subscriptions are released on shutdown or adapter restart.

## 8. Data Flows

### 8.1 Batch arrangement

1. Svelte requests the latest screens, layouts, and Window Catalog.
2. The user creates ephemeral zone assignments in UI state.
3. Svelte sends the layout ID, target screen ID, and assignments to one typed command.
4. The Arrange Orchestrator refreshes the catalog and validates every ID, zone, capability, workspace, and target screen.
5. If preflight fails, no window is changed and structured errors identify stale assignments.
6. The orchestrator snapshots rectangle, minimized state, and relevant window state for every assigned window.
7. Minimized windows are restored, then windows are moved and resized sequentially.
8. The orchestrator reads back final bounds and compares them with decoration-aware target bounds.
9. On success, Svelte receives a result for every assigned window.
10. On partial failure, already changed windows are restored on a best-effort basis and the result distinguishes operation failures from rollback failures.

The orchestrator serializes arrangement operations so a second command cannot race the first.

### 8.2 Modifier-assisted snap

1. Platform events indicate that the configured modifier is held during a window drag.
2. Snap Controller confirms that snap is enabled, the window is eligible, and the pointer is on the active target screen.
3. Layout Engine supplies pixel zone rectangles for the current screen work area.
4. Snap Controller performs point-in-zone hit testing in Rust.
5. Overlay Controller renders zones and the current highlight.
6. A valid drop hides the overlay and calls `move_resize_window` once.
7. Programmatic movement events for that operation are tagged or suppressed so they cannot start a second snap cycle.
8. The final bounds are verified and any failure is sent to the UI notification queue and local log.

## 9. Failure Handling

### 9.1 Safety rules

- Validate all batch assignments before changing any window.
- Never reuse a stale window ID without revalidation.
- Never change an unassigned window.
- Serialize window-arrangement operations.
- Suppress self-generated move events.
- Hide and destroy overlays on cancellation, adapter error, and shutdown.
- Bound retries; no operation loops indefinitely.

### 9.2 User-visible failures

| Condition | Behavior |
|---|---|
| Wayland or missing X11 control | Disable arrangement and show `X11 required` guidance. |
| Required EWMH capability absent | Disable the affected feature and show it in `Settings > System Status`. |
| Assigned window closed or changed | Reject the batch before movement and mark the stale card. |
| Window manager rejects movement | Report the affected window and attempt rollback. |
| Application enforces size constraints | Keep the closest accepted bounds and report the limitation. |
| Target screen disconnects | Cancel the operation, hide overlay, and request a new screen selection. |
| Workspace changes | Clear assignments, cancel active snap, and refresh the catalog. |
| Config parse or validation failure | Preserve the invalid file, load the newest valid backup, or use defaults. |
| Autostart setup fails | Keep the app usable and show a settings-level error. |

Transient notifications use concise toasts. Persistent environment problems use an inline status panel with a concrete recovery action.

### 9.3 Logging and privacy

Logs are local and rotated. They include component, operation, timing, capability status, and opaque window identifiers. Window titles are redacted by default because they may contain sensitive document or website names. The MVP makes no network requests and sends no telemetry.

## 10. Persistence and Security

Configuration is stored under `${XDG_CONFIG_HOME:-~/.config}/grid-screen/` as versioned JSON. Writes use a temporary file, flush, validation read-back, and atomic rename. A small rotating backup set is retained.

Only the Rust core accesses the filesystem and native window APIs. The webview receives a narrow command allowlist and cannot invoke shell commands, arbitrary filesystem APIs, or remote HTTP. The content security policy permits local packaged assets and Tauri IPC only. User-provided layout names and platform-provided application titles are rendered as text, never injected as HTML.

The application runs with normal user privileges and never requests root access.

## 11. Accessibility

- Every draggable window card and zone is keyboard focusable.
- Assignment has a keyboard menu alternative.
- Divider movement supports arrow keys, with a documented fine-adjust modifier.
- Focus order follows navigation, window list, canvas, then primary action.
- Zone identity is conveyed by label and border, not color alone.
- Status and errors use appropriate live regions without repeatedly announcing pointer movement.
- All controls expose English accessible names and visible focus indicators.
- Motion is minimal and respects reduced-motion preferences in the webview.

## 12. Performance Targets

- Idle CPU below 1% on the supported test machine.
- No continuous cursor or geometry polling while idle.
- Batch arrangement of three supported windows completes within one second after activation.
- A valid snap completes within 150 ms of mouse release.
- Zone hit testing and overlay rendering remain responsive for the preset-derived layout sizes supported by the UI.
- An eight-hour idle soak and repeated arrange/snap test show no unbounded memory, X11 resource, thread, or subscription growth.

These targets are measured in release builds. Test reports record hardware, desktop environment, window manager, and display configuration.

## 13. Testing Strategy

### 13.1 Unit and property tests

- Partition-tree validation and serialization.
- Divider ratio changes and minimum-size constraints.
- Deterministic pixel rounding.
- Gap, margin, and work-area calculations.
- Assignment uniqueness and stale-ID rejection.
- Snap Controller state transitions and cancellation.
- Config validation, migration, atomic write, backup, and recovery.
- Property tests generate thousands of random valid trees and assert bounds, complete un-gapped coverage, and absence of overlap.

### 13.2 Core integration tests

A `MockPlatformAdapter` covers:

- Successful batch arrangement.
- Minimized-window restoration.
- Windows originating on another screen.
- Preflight failure with zero mutations.
- Mid-batch platform failure.
- Successful and failed rollback.
- Modifier release and invalid drop.
- Self-generated move-event suppression.
- Workspace change and screen disconnect.
- Adapter capability degradation.

### 13.3 X11 integration tests

Automated X11 tests run with a real EWMH-compatible window manager and controlled test windows. They verify enumeration, eligibility filtering, restore, move/resize, read-back geometry, screen work areas, event subscriptions, and overlay input pass-through.

### 13.4 UI tests

- Onboarding and explicit autostart consent.
- Target-screen and layout selection.
- Window-card drag assignment and reassignment.
- Keyboard assignment alternative.
- Empty and stale window states.
- Layout preset selection and divider adjustment.
- Structured success, partial failure, and persistent environment errors.
- System tray state reflected when the configuration window reopens.

### 13.5 Manual compatibility matrix

The release candidate is exercised on representative X11 sessions for GNOME Xorg, KDE Plasma X11, and Xfce, including mixed resolutions, different window decorations, minimized windows, screen disconnects, and common applications.

## 14. Acceptance Criteria

The MVP is ready when all of the following hold:

1. A first-time usability test participant can choose a preset, assign three running windows, and arrange them in less than 60 seconds without external instruction.
2. Every supported, resizable test window lands within two physical pixels of its decoration-adjusted target bounds.
3. No test operation controls a window outside the current workspace or a window other than the one selected.
4. Batch arrangement completes within one second and a valid snap within 150 ms on the recorded reference environment.
5. Idle CPU remains below 1%, and no continuous idle polling is present.
6. Closing windows, disconnecting a screen, changing workspace, corrupting config, or receiving a window-manager rejection does not crash the process.
7. Partial batch failure produces a per-window result and attempts rollback.
8. The configuration window can close and reopen while tray and snapping remain functional.
9. Keyboard users can assign windows and adjust dividers without drag-and-drop.
10. Wayland and missing X11 capabilities produce an explicit unsupported state rather than silent malfunction.

## 15. Future Extension Boundary

Future operating systems implement `PlatformAdapter` and declare capabilities. Product logic, layout serialization, assignment validation, and most UI components must not branch on operating-system names. Platform-specific UX is introduced only when a capability cannot be represented by the shared contract, such as compositor-specific Wayland integration or platform permission onboarding.

The next platform is intentionally undecided. No MVP design choice assumes that Windows, macOS, or Wayland follows Linux X11.
