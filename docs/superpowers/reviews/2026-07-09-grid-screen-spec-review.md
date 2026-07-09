# Spec Review: Grid Screen

**Date:** 2026-07-09
**Spec:** `docs/superpowers/specs/2026-07-09-grid-screen-design.md`
**Tech Stack:** Rust, Tauri 2.x, Svelte 5
**Reviewers:** rust-engineer, performance-engineer, security-auditor, devops-engineer, product-manager

---

## Verdict: **Needs Changes**

**Summary:** The spec defines a coherent product vision and a sound high-level architecture, but the critical subsystems — drag loop mechanism, threading model, overlay rendering pipeline, Tauri security model, and product positioning — are described at the "what" level without the "how." These are implementation-defining decisions that must be resolved before code is written.

**Finding counts:** 12 Critical, 12 Important, 8 Minor

---

## Consolidated Findings

### Architecture & Threading (Rust backend)

#### Critical

1. **Drag loop mechanism is undefined — risk of 100% CPU or broken UX**
   - Section: DragDetector, lines 118–123
   - Issue: Both the technical and performance reviewers flagged this. The spec says "polls cursor each frame" but never defines the frame mechanism. A busy-loop would saturate a CPU core. The `subscribe_window_move_events` callback signature is entirely unspecified (no type signature, no threading contract, no cleanup).
   - Impact: The core interaction — dragging a window — either burns battery at 100% CPU or doesn't work at all. On Windows, `SetWinEventHook` requires a message pump on the hook thread. On X11, `ConfigureNotify` requires its own event loop. Neither is compatible with a simple callback model.
   - Recommended Fix: Redesign as an event-driven stream. `PlatformApi` returns a channel receiver (`mpsc::Receiver<WindowMoveEvent>`). Each platform impl spawns its own dedicated event-loop thread and sends through the channel. DragDetector consumes the channel on its own thread. Add explicit cleanup on unsubscribe.

2. **Self-triggered drag detection loop**
   - Section: DragDetector, lines 118–123
   - Issue: After snapping a window via `move_window()`, that move generates another drag event. This creates a spurious re-detection cycle or infinite loop.
   - Impact: Core interaction is broken — windows bounce or snap repeatedly.
   - Recommended Fix: Add `snap_in_progress: bool` to `DragState`. Discard events for the snapped window handle until the next idle cycle.

3. **Single `Arc<Mutex<AppState>>` creates contention on the hotpath**
   - Section: State Management, lines 230–244
   - Issue: Both technical and performance reviewers flagged this. A single mutex held during zone iteration + overlay rendering blocks everything: drag processing, config IPC, monitor polling. The concurrent access model is entirely undefined — which threads exist, what they hold, when they lock.
   - Impact: Config window freezes during a drag. Potential deadlocks between drag loop and IPC. Priority inversion.
   - Recommended Fix: Split into granular locking. `active_layout` + `monitors` behind `ArcSwap` (lock-free reads for hotpath). `drag_state` behind a separate `Mutex`. `AppState` behind `RwLock` for config mutations only. Overlay rendering must happen outside any lock. Define explicit thread topology: main thread (Tauri + tray), platform thread (event loop), drag processor thread (channel consumer).

4. **Overlay click-through behavior not specified — make-or-break for UX**
   - Section: ZoneOverlay & Platform-Specific Details
   - Issue: An overlay window that consumes mouse events captures the drag — the user's release goes to the overlay, not the window being dragged. For Windows this requires `WM_NCHITTEST → HTTRANSPARENT`. For X11 this requires `XShapeCombineRectangles` with empty input shape. Neither is mentioned.
   - Impact: Dragging windows simply doesn't work. The overlay eats the mouse release event.
   - Recommended Fix: Add explicit input passthrough requirements to `PlatformApi::create_overlay_window` semantics and the platform-specific details sections. Windows: `WS_EX_TRANSPARENT` + `WM_NCHITTEST → HTTRANSPARENT`. X11: empty input shape via `XShape` extension.

5. **Overlay rendering pipeline from tiny-skia to platform window is unspecified**
   - Section: ZoneOverlay & Platform-Specific Details
   - Issue: tiny-skia renders to a CPU-side `Pixmap` (`Vec<u32>`). The spec never defines how this buffer gets onto the overlay window surface. Windows needs `UpdateLayeredWindow` with DIB format conversion. X11 needs `XCreatePixmap` + `XCopyArea` with byte-order awareness.
   - Impact: The visual overlay — a core feature — cannot be implemented without defining the blit path.
   - Recommended Fix: Add `fn overlay_present(handle, pixels: &[u8], w, h)` to `PlatformApi`. Specify pixel format: premultiplied BGRA for Windows, native-endian ARGB for X11. ZoneOverlay renders into tiny-skia Pixmap, extracts pixels, calls overlay_present each frame.

6. **Drag start heuristic undefined**
   - Section: DragDetector, lines 120–121
   - Issue: What constitutes "drag start"? A single move event could be a programmatic window change, Aero Snap, or a compositor animation. Without distinguishing user-initiated drags, the overlay fires spuriously. Also: pre-allocated pixel buffers vs per-frame allocation not specified (at 4K, per-frame alloc = 1.9 GB/s of traffic).
   - Impact: Overlay flickers randomly. Memory churn degrades performance.
   - Recommended Fix: Define drag start as move events that begin while the mouse button is held. Use `GetAsyncKeyState(VK_LBUTTON)` on Windows, `XQueryPointer` on X11. Introduce 5px movement threshold before showing overlay. Pre-allocate and reuse pixel buffers. Use dirty-rect rendering.

7. **Monitor detection via polling is architecturally inferior**
   - Section: MonitorManager, line 103
   - Issue: Both platforms provide native event-driven notification for display changes (`WM_DISPLAYCHANGE` on Windows, `RRScreenChangeNotify` on X11). Polling every 2 seconds burns CPU for no benefit and introduces up to 2-second layout-switching latency.
   - Impact: Poor hotplug UX. Unnecessary battery drain on laptops.
   - Recommended Fix: Replace polling with native event subscription. Add `subscribe_display_change_events` to `PlatformApi`. Keep 30s polling as safety-net fallback only.

### Security & Tauri IPC

8. **Tauri IPC capability model and CSP undefined**
   - Section: Architecture, IPC, Frontend
   - Issue: Both technical and security reviewers flagged this. Tauri 2.x uses deny-by-default capabilities. Without explicit capability configuration, the webview cannot invoke backend commands. More critically, if misconfigured (e.g., wildcard `shell:allow-execute`), an XSS in the webview becomes full backend compromise via the config file (layout names stored unsanitized → stored XSS → arbitrary IPC calls).
   - Impact: Build fails at compile time without capabilities. Security risk: config file stored-XSS → backend RCE.
   - Recommended Fix: Define exact Tauri capabilities: `core:default`, `tray:default`, custom `gridscreen:default`. Deny: `shell:`, `http:`, broad `fs:`. Define CSP: `default-src 'self'; script-src 'self'; connect-src 'self' ipc: https://ipc.localhost`. List all IPC command signatures with input validation per command.

9. **Input validation is undefined**
   - Section: ConfigStore, LayoutManager
   - Issue: The spec says config is "validated" but never says against what. Layout names are free-text user input → stored XSS vector. Zone coordinates unchecked → negative values/NaN/Infinity passed to OS APIs. Zone count unbounded → 100k zones in a crafted config crashes the app.
   - Impact: Stored XSS. OS API misuse. DoS via malicious config file.
   - Recommended Fix: Define per-field validation. Zone: x/y in [0, 65535], w/h > 0, within monitor bounds. Names: max 64 chars, sanitize for HTML special chars. Max 64 zones per monitor. Add `schema_version` field for future format migration.

### Product & UX

10. **No problem statement or success criteria**
    - Section: Overview
    - Issue: The spec describes *what* the product does but never *why*. What pain point does it solve? No metrics or targets for when v1 is "done." No competitive analysis (PowerToys FancyZones on Windows, tiling WMs on Linux). No user research confirming demand from "general, non-technical users" for a power-user-adjacent feature.
    - Impact: Cannot evaluate whether design decisions serve the right problem. Risk of building something nobody installs.
    - Recommended Fix: Add a brief Problem Statement section. Add 2-3 measurable success criteria. Add a Competitive Positioning subsection explaining differentiation from PowerToys FancyZones and Linux alternatives.

11. **First-run experience and user-facing error states undefined**
    - Section: Frontend, Error Handling
    - Issue: For a "GUI-first, install-and-use" product, the install-to-first-snap journey is undescribed. What does the user see on first launch? How do they discover that dragging windows now does something? Error handling describes engineering fallbacks but never what the *user* sees.
    - Impact: Non-technical users won't understand how to use the product or what went wrong.
    - Recommended Fix: Add an Onboarding section: first-launch wizard (2 steps: pick a default layout or draw one), tray notification confirming activation. Add user-facing error messages for key failure modes (config corruption = "Layout reset to default", Wayland = "Some features limited on this display system").

12. **Accessibility absent**
    - Section: Frontend, Overview
    - Issue: Nothing in the spec addresses screen readers, keyboard alternatives, color contrast, or DPI scaling — for a GUI-first consumer app targeting non-technical users.
    - Impact: Excludes users with disabilities. Poor experience on high-DPI displays.
    - Recommended Fix: Add Accessibility section. Minimum: keyboard focus in config UI, ARIA labels on zone editor, WCAG AA color contrast for overlays and UI, proper DPI scale handling in zone calculations.

### Operations & Delivery

#### Important

13. **No packaging, distribution, or CI/CD plan**
    - Section: (Missing)
    - Issue: The spec targets non-technical users but has no plan for installers, code signing, auto-updates, or CI pipelines. A cross-platform Rust/Tauri project that won't compile on the wrong OS needs build matrix CI to prevent regressions.
    - Recommended Fix: Add Distribution section: GitHub Releases with platform-specific installers (MSI/NSIS for Windows, AppImage/deb for Linux), Tauri updater for auto-updates, semver versioning. Add CI section: GitHub Actions matrix for ubuntu-latest + windows-latest.

14. **No logging or crash reporting strategy**
    - Section: Error Handling
    - Issue: Both process and technical reviewers flagged this. A silent system tray app with no logs is impossible to debug in the field. No error type architecture defined (thiserror? anyhow?).
    - Impact: Field issues are undiagnosable. Users can't report bugs effectively.
    - Recommended Fix: Add logging strategy: `tracing` crate with file appender at config dir, rotation (3 files × 1MB), "View Logs" tray menu item. Use `thiserror` for domain errors.

15. **MonitorFingerprint too rigid — DPI/rotation changes break layouts**
    - Section: MonitorManager, LayoutManager
    - Issue: Fingerprint is a hash of exact resolution and position. Changing DPI scale (100%→150%), rotating a display, or slightly repositioning monitors in OS settings silently invalidates all saved layouts. The `Monitor` struct stores `dpi_scale` but zones are stored as raw pixels — meaning DPI changes make all zone coordinates wrong.
    - Impact: Users lose layouts after routine OS settings changes. Silent fallback to default is confusing.
    - Recommended Fix: Store zones in fractional coordinates (0.0–1.0 relative to monitor dimensions). Use fuzzy fingerprinting: match on monitor EDID/serial + topology, not exact pixel coordinates. Notify user when fuzzy match is used.

16. **Zone gap/margin semantics undefined**
    - Section: LayoutManager, line 112
    - Issue: Zone has `gap` and `margin` fields with no definition. Are gaps between adjacent zones? Are margins from monitor edge or zone edge? Do they reduce effective zone size or offset position? Both overlay rendering and snap logic depend on getting these right.
    - Impact: Zone math cannot be implemented without defining these interactions.
    - Recommended Fix: Define gap = space between adjacent zones within a monitor (borders inset). Margin = space between zone and monitor edge. Per-zone values override Settings defaults.

### Additional Important Findings

17. Show overlay only on the active monitor during drag, not all monitors (wasteful GPU/CPU on multi-monitor setups)
18. Config backup strategy needs specifics: rotation count, naming convention, write-verify-before-rotate pattern
19. `subscribe_window_move_events`: Pause/resume during active drag needs defined behavior (cancel drag immediately, do not snap)
20. Layout Editor UX needs more detail: grid resolution, zone overlap rules, multi-monitor display mode, zone creation interaction model
21. Monitor polling side-channel: replace with native events, keep polling as safety-net only
22. Wayland detection must be explicit: attempt Wayland socket connect, determine XWayland vs native per-window
23. Config file needs integrity verification (schema_version field, structural validation)
24. Driver/overlay anti-tampering: periodic z-order verification, window property monitoring

### Minor Findings

- Architecture diagram shows "Linux X11 + Wayland" but Wayland is out of scope — should show dashed/Phase 2
- "fingerprint" term overloaded — rename to `MonitorArrangementId`
- Zone count cap needed (64 per monitor)
- HashMap vs Vec microbenchmark for small monitor counts
- Sequential startup could overlap config load with monitor enumeration
- No `cargo audit`/`cargo deny` in testing strategy for dependency vulnerabilities
- Pre-allocate overlay windows for max expected monitor count (4) instead of create/destroy on hotplug
- Add frame-timing instrumentation to dev builds (FPS counter, frame-time histogram)

---

## Strengths

- Clean architecture separation: real-time Rust backend vs config-only webview — eliminates an entire class of latency/reliability problems
- PlatformApi trait abstraction enables mock testing and clean Wayland phase 2
- Error handling is pragmatic: graceful degradation, corrupt config fallback without overwrite, no cascading failures
- Wayland Phase 1/Phase 2 strategy acknowledges real platform constraints
- No network surface, no auth needed, no remote content — correct for a desktop app
- Correctly scoped "Not In Scope" section with clear YAGNI discipline
- Appropriate JSON file config for a single-user desktop app
- Tray + pause toggle + auto-start shows understanding of user mental model
