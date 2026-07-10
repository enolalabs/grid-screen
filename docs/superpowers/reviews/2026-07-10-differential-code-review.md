# Differential Code Review — 2026-07-10

## Scope and method

- **Baseline:** `HEAD~5` (`57cd5c8`)
- **Reviewed range:** `57cd5c8..18940d0` (21 files; 7,177 added and 427 removed lines)
- **Primary risk areas:** new X11/Hyprland backends, window handles, overlay creation, and Tauri capabilities.
- **Repository size:** small (about 40 source/test files); reviewed all changed implementation files and relevant callers/history.

## Executive summary

Six issues were found: two release-blocking regressions and four high-impact functional defects. The new backends compile, but the reviewed paths are not covered by automated tests against their platform protocols.

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 4 |
| Warning | 2 |
| Info | 0 |

## Findings

### High — X11 window enumeration drops EWMH windows

**Location:** `src-tauri/src/platform/linux.rs:652-668`  
**Introduced by:** `0b743ef`

`get_window_title` reads `_NET_WM_NAME` while requiring the property type `STRING`. EWMH specifies this property as `UTF8_STRING`; when the requested type differs, the X server returns no property value. The fallback only runs after that empty value and requests legacy `WM_NAME`, which is not guaranteed to be populated.

`enumerate_windows` then discards every window with an empty title at `linux.rs:230-232`. On a normal EWMH desktop, this can make the new backend report no usable windows. Use the `UTF8_STRING` atom (or request `Any`) for `_NET_WM_NAME`, preserve the legacy fallback, and add an Xvfb integration test with a UTF-8 EWMH title.

### High — Hyprland uses a hash as an address selector

**Location:** `src-tauri/src/platform/hyprland.rs:158-161`, `175-191`  
**Introduced by:** `0b743ef`

The backend exposes `simple_hash(window.address)` as `WindowHandle`, then reconstructs `address:0x<hash>` when it calls `hyprctl`. A Hyprland address is already a hexadecimal pointer-like identifier; its DJB2 hash is a different number. Therefore both `movewindowpixel` and `resizewindowpixel` target a nonexistent address and their errors are discarded.

This breaks snapping for every Hyprland window. Store a reversible handle instead (for example parse the native hexadecimal address into `u64`, with explicit validation) or maintain a synchronized `WindowHandle -> address` map. Propagate/log command failures and test the selector sent to `hyprctl`.

### High — Hyprland cannot detect a drag without XWayland

**Location:** `src-tauri/src/platform/hyprland.rs:81-83`, `97-111`, `209-210`, `226-296`  
**Introduced by:** `0b743ef`

`is_mouse_down_fallback` is only constructed and loaded; there is no `store` anywhere in the backend. If `$DISPLAY` is absent or the X11 QueryPointer attempt fails, `check_mouse_button` always returns `false`. The event loop consequently emits no `DragStart` and cannot snap native Hyprland windows.

Either obtain pointer-button state from a supported Hyprland input mechanism, or explicitly limit this backend to XWayland and select the X11 backend. Add a test for the no-`DISPLAY` path so the advertised fallback cannot silently remain inert.

### High — ARGB overlay window is created with the wrong colormap

**Location:** `src-tauri/src/platform/linux.rs:635-649`, `500-529`  
**Introduced by:** `0b743ef`

When a 32-bit ARGB visual is found, `find_argb_visual` returns that visual together with `screen.default_colormap`. The default colormap belongs to the root visual (normally 24-bit), not the discovered 32-bit visual. `CreateWindow` therefore receives a visual/colormap mismatch and the server rejects the request with `BadMatch`; because the request is not checked, this becomes a missing overlay later rather than a reported error.

Create and retain a colormap for the selected ARGB visual (or use the root visual/depth consistently). Check asynchronous X11 errors before returning an `OverlayHandle`, and cover transparent overlay creation under Xvfb.

### Warning — Capability hardening test is broken and privilege expansion is unreviewed

**Location:** `src-tauri/capabilities/gridscreen.json:8-10`, `src-tauri/tests/security_smoke.rs:12-29`  
**Introduced by:** `c370c72`

The capability change replaces the expected tray permission and adds `shell:allow-open` plus `updater:default`, but the security allowlist still rejects both the new tray name and all `shell:` permissions. `cargo test --manifest-path src-tauri/Cargo.toml --test security_smoke` fails immediately.

This is a release-blocking test regression. Also decide whether the frontend truly needs those two broad permissions, document the rationale/scope, and update the security test to assert the deliberately approved, least-privilege set rather than simply weakening it.

### Warning — The 30-second monitor safety net ignores geometry-only changes

**Location:** `src-tauri/src/monitor_manager.rs:34-40`  
**History:** condition changed in `c370c72`; surrounding polling predates this review range.

The polling branch compares only monitor IDs. `current.len() != prev_ids.len()` is redundant because `prev_ids.len()` is always the previous monitor count. Resolution, scale, position, or primary-monitor changes keep the same IDs and are never stored. If the event subscription misses such a change, layouts and hit testing continue using stale geometry indefinitely.

Compare the complete monitor snapshots (or a geometry fingerprint), and add a test that changes a monitor's dimensions while preserving its ID.

## Test coverage and validation

- `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast -q` was attempted as the full Rust suite; the focused security test below identifies a failure.
- `cargo test --manifest-path src-tauri/Cargo.toml --test security_smoke -- --nocapture` **failed**: `test_capabilities_only_permit_expected` rejects `core:tray:default` before also reaching the new shell/updater permissions.
- `npx vitest run` could not run because the workspace has no local `node_modules/.bin/vitest` and the environment's `npx` shim is broken (`ENOENT` for its bundled `lib` directory).
- No automated tests exercise `HyprlandPlatformApi`, the X11 `_NET_WM_NAME` type, ARGB colormap creation, or the no-XWayland drag path.

## Blast radius and prioritization

`PlatformApi::move_window` is invoked by the snap-consumer thread in `src-tauri/src/lib.rs:203`, so the irreversible Hyprland selector issue prevents the core snap action. `PlatformApi::subscribe_window_move_events` feeds the sole `DragDetector` event loop, so the missing fallback prevents the entire native-Hyprland flow. The X11 title and overlay findings affect the two core user-visible Linux operations: detecting candidate windows and showing the drag overlay.

## Recommended order

1. Fix the failing capability test and explicitly approve or remove the newly added permissions.
2. Make the Hyprland window handle reversible and implement/limit the no-XWayland drag path.
3. Correct the X11 `UTF8_STRING` property request and ARGB colormap lifecycle.
4. Fix geometry comparison in the monitor polling safety net.
5. Add protocol-level tests (Xvfb for X11; mocked `hyprctl`/IPC for Hyprland) before enabling the new backends by default.

## Coverage limits

This was a source and history review. It did not run a live X11 server or Hyprland compositor, so the X11 and Hyprland protocol findings are based on the API contracts and direct control-flow analysis rather than a compositor-session reproduction.
