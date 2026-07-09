# Plan Review: Grid Screen

**Date:** 2026-07-09
**Plan:** `docs/superpowers/plans/2026-07-09-grid-screen-implementation.md`
**Spec:** `docs/superpowers/specs/2026-07-09-grid-screen-design.md`
**Tech Stack:** Rust, Tauri 2.x, Svelte 5
**Reviewers:** rust-engineer, performance-engineer, security-auditor, devops-engineer, product-manager

---

## Verdict: **Needs Changes**

**Summary:** The plan is structurally sound with correct module decomposition, threading model, and test-mocking strategy. However, it has concurrency-model conflicts, missing performance validation tasks, underspecified integration points, and critical UX gaps (i18n, error bridging). These are fixable with targeted additions.

**Finding counts:** 6 Critical, 21 Important, 10 Minor

---

## Consolidated Findings

### Critical (Must Fix)

1. **C1: ArcSwap vs RwLock conflict in `active_layout` storage** (Technical)
   - Task 5 defines `LayoutManager` with `RwLock<Option<Layout>>` for `active_layout`. Task 8 defines `AppState` with `ArcSwap<Vec<Layout>>` for `active_layouts`. These are fundamentally incompatible concurrency primitives — which one owns the canonical layout?
   - **Fix:** Unify on `ArcSwap` in `AppState`. Remove `RwLock` from `LayoutManager::active_layout` and make `LayoutManager` a stateless code layer that reads/writes through `AppState`'s `ArcSwap`. `LayoutManager::get_zones()` reads from `ArcSwap::load()`; `LayoutManager::activate()` does `ArcSwap::store(Arc::new(...))`.

2. **C2: `SnapEvent` type undefined** (Technical)
   - Task 7's DragDetector sends `SnapEvent` through mpsc, but `SnapEvent` is not declared in Task 2's type list, and Task 8 never references it as a consumer.
   - **Fix:** Add `pub struct SnapEvent { pub window_handle: WindowHandle, pub zone_rect: Rect }` to Task 2's `types.rs`. In Task 8, wire the `snap_sender` channel receiver to a handler that calls `PlatformApi::move_window(handle, zone_rect)`.

3. **C3: No performance validation tasks for spec budgets** (Technical, Performance)
   - Spec requires ≥60 FPS, <15% CPU, <60MB idle, <500ms startup. Zero tasks measure or enforce any of these.
   - **Fix:** Add a task (Task 16) for performance instrumentation: `tracing` spans on drag loop, FPS counter in dev build overlay, simulated drag benchmark (30s, 64 zones), and startup-time measurement. Add CI assertion that benchmark meets budgets.

4. **C4: Zero frontend test specifications** (Technical)
   - Tasks 11, 12, 13 contain no test requirements — no component render tests, no IPC mock tests, no keyboard event tests, no ARIA snapshot validation.
   - **Fix:** Add test steps to Tasks 11 and 12 using Vitest + `@testing-library/svelte`: component renders, IPC mock interactions, keyboard navigation tests, and confirmation dialog tests.

5. **C5: i18n is a UI stub with no plumbing** (Product)
   - Task 12 adds a language dropdown but there is no task to integrate an i18n library, extract strings, or translate the UI into Vietnamese. A dropdown that changes nothing is worse than no dropdown.
   - **Fix:** Add an i18n subtask to Task 13: integrate `svelte-i18n` or `typesafe-i18n`, extract all user-facing strings to JSON dictionaries (en.json, vi.json), and translate all strings to Vietnamese.

6. **C6: No backend-to-frontend error bridging** (Product)
   - Spec requires config corrupt → tray notification, Wayland → notification, save fail → retry notification. Tasks 1–9 are backend-only. Task 13 builds a toast system but nothing wires backend errors to any notification channel.
   - **Fix:** Add a task: create a `UserNotifier` module in Rust backend with `notify_error(message)`, `notify_warning(message)`, `notify_info(message)`. Wire it to Tauri events (`user-notification`). Frontend listens for this event and maps to toast notifications. Wire all error paths (Task 3 corruption, Task 8 save fail, etc.) to call `UserNotifier`.

### Important (Should Fix)

#### Architecture & Types

7. **I1: `subscribe_window_move_events` return type is implicit** (Technical)
   - Task 2's trait returns `mpsc::Receiver` without type parameter. Task 7 consumes `mpsc::Receiver<WindowMoveEvent>`. Can diverge silently.
   - **Fix:** Make type parameter explicit in Task 2: `fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent>`.

8. **I2: Zone `effective_rect()` conversion locus ambiguous** (Technical)
   - Task 5 includes a test for fractional-to-pixel conversion, but that method (`effective_rect()`) is defined on `Zone` in Task 2. Test should live in Task 2, not Task 5.
   - **Fix:** Move the `fractional_to_pixel_conversion` test from Task 5 to Task 2 where `Zone::effective_rect()` is defined.

9. **I3: ZoneOverlay render loop mechanism undefined** (Technical, Performance)
   - Task 6 says "calls overlay_present() per frame" but no frame loop is defined. Event-driven callback? Sleep-loop at 60 FPS? mpsc-driven?
   - **Fix:** Document in Task 6: the overlay render thread uses `std::sync::mpsc::Receiver<OverlayCommand>` and blocks on `rx.recv()`. On receiving an `Update` command, renders and calls `overlay_present()`. On receiving a `Hide` command, destroys windows and parks.

10. **I4: Safety-net polling mechanism underspecified** (Technical)
    - Task 4 says "30-second safety-net polling thread" but doesn't specify the API called, comparison method, or shutdown.
    - **Fix:** Specify: "Every 30s, calls `enumerate_monitors()`, computes arrangement ID, compares with cached value. If different, sends `DisplayChangeEvent` through the same channel. Thread shuts down on channel drop."

11. **I5: No DPI-awareness in zone rendering** (Technical)
    - `Monitor` type has `dpi_scale` but `Zone::effective_rect()` takes `Rect` (pixel coords), not the monitor's DPI. Overlays and snap targets will be misaligned on scaled displays.
    - **Fix:** Thread `dpi_scale` through `effective_rect(monitor: &Monitor, dpi_scale: f64)` and apply the scale factor when computing draw coordinates in ZoneOverlay.

12. **I6: Auto-start backend missing** (Technical)
    - Task 12's Settings screen has an auto-start checkbox, but no task writes the OS-specific autostart entry (Windows Registry, Linux `.desktop` file in `~/.config/autostart`).
    - **Fix:** Add `fn set_autostart(enabled: bool) -> Result<()>` to `PlatformApi` trait in Task 2, with OS implementations. Wire the Settings save button to call it.

#### Performance

13. **I7: Pre-allocated buffer vs memory budget conflict** (Performance)
    - 4K monitor Pixmap = 33 MB. 4 monitors = 133 MB for pixel buffers alone, exceeding the <100 MB drag budget before any other memory.
    - **Fix:** Document single-Pixmap reuse strategy in Task 6: allocate one Pixmap for the current monitor only. When cursor crosses monitors, reallocate for the new monitor (the old one is dropped). Add a note: peak memory during multi-monitor drag may transiently reach 133 MB while reallocating — track with `dhat`.

14. **I8: CSP not assigned to any task** (Security)
    - Spec requires strict CSP. Task 1 configures Tauri capabilities but CSP is not explicitly mentioned in any task step.
    - **Fix:** Task 1's `tauri.conf.json` already includes the CSP in the `security.csp` field — verify this is correct in the implementation step. Add an explicit verification substep: "Inspect the dev build to confirm CSP headers are served" using browser devtools.

15. **I9: Input validation missing finiteness and HTML escaping at Rust layer** (Security)
    - Task 3 validates zone bounds and names but doesn't check `is_finite()` for `NaN`/`Infinity` on f64 values, and defers HTML escaping entirely to Svelte. Spec requires Rust-side escaping too.
    - **Fix:** Add `if !zone.x.is_finite() || !zone.y.is_finite() || !zone.width.is_finite() || !zone.height.is_finite()` to `validate_zone()`. Add HTML entity escaping of `<>&"'` in zone names at save time in ConfigStore.

16. **I10: `cargo audit` and `cargo deny` missing from CI** (Security, Process)
    - Task 14 has `fmt → clippy → test → build` but no dependency scanning.
    - **Fix:** Add `cargo audit` and `cargo deny check` steps to Task 14's CI matrix, after `test` and before `build`.

17. **I11: Tauri updater not configured** (Process)
    - Spec requires Tauri updater for auto-updates. Tasks 14 and 15 don't include the updater plugin or configuration.
    - **Fix:** Add to Task 15: enable `tauri-plugin-updater` in Cargo.toml, add updater config to `tauri.conf.json` (endpoints pointing at GitHub Releases JSON feed, public key for signature verification).

18. **I12: Windows MSI bundler missing** (Process)
    - Task 15 mentions NSIS only. Spec says "NSIS/MSI." MSI is needed for enterprise deployment.
    - **Fix:** Add MSI target to Task 15's bundler config: `"msi": {}` in the windows bundle section.

19. **I13: No crash reporting mechanism** (Process)
    - Desktop app needs a panic hook capturing backtraces to the log directory.
    - **Fix:** Add a panic hook in Task 8's `setup_logging()`: `std::panic::set_hook(Box::new(|info| { tracing::error!("PANIC: {:?}", info); std::process::abort(); }))`.

#### UX

20. **I14: No zone deletion confirmation dialog** (Product)
    - Task 11 uses `confirm()` in JavaScript but spec requires "Confirmation dialogs for destructive actions." The built-in `confirm()` is unstyled; should match the app design.
    - **Fix:** Replace `if (confirm(...))` with a custom styled confirmation dialog component or ensure the toast/overlay design system is applied to confirmation prompts.

21. **I15: Layout Editor error states undefined** (Product)
    - Task 11 describes happy path only. What does user see when zone creation fails, save fails, or grid config is invalid?
    - **Fix:** Add error-state UI to Task 11: save-fail shows red toast via the notification system (Task 13), zone creation failure shows inline error text on the canvas.

22. **I16: WCAG AA color contrast verification missing** (Product)
    - Spec requires WCAG AA for overlays, but no task tests this.
    - **Fix:** Add to Task 11 or Task 13: a manual QA checklist item verifying that the accent color (`#7C3AED`) meets 4.5:1 contrast ratio against both light and dark backgrounds. Provide the contrast ratio calculation.

23. **I17: High-DPI verification unassigned** (Product)
    - Plan summary acknowledges this gap. Must assign a task.
    - **Fix:** Add to Task 15 (Distribution & Polish): a "High-DPI QA" manual checklist — test at 100%/125%/150%/200% scaling, verify zone rendering is crisp, text readable, overlays correctly sized.

24. **I18: Log rotation is DAILY, spec says size-based** (Process)
    - Task 8 configures DAILY rotation. Spec says "3 files × 1MB." These are different rotation strategies.
    - **Fix:** Align with spec — use `tracing_appender::rolling::Builder::new().rotation(Rotation::NEVER).max_file_size(1_000_000).max_log_files(3)` (size-based, not daily).

### Minor Fixes

- **M1:** Move `effective_rect()` test from Task 5 to Task 2 where `Zone` is defined
- **M2:** Add 5px threshold boundary test to Task 7 (exactly 4px → ignored, 6px → detected)
- **M3:** Add CI caching for Rust builds in Task 14 (`Swatinem/rust-cache@v2`)
- **M4:** Specify Svelte state approach explicitly: use `$state` runes for component-local, `svelte/store` for cross-component shared state
- **M5:** Set config file permissions to `0o600` in ConfigStore initialization (Task 3)
- **M6:** Persist "onboarding completed" flag in `AppSettings` (Task 2 types, saved via ConfigStore)
- **M7:** Add icon asset generation to Task 9 (16×16, 32×32, 48×48, 128×128 variants)
- **M8:** Startup cleanup of orphaned `.tmp` files in ConfigStore::load() (Task 3)
- **M9:** Security verification smoke test: parse capabilities JSON, assert only 4 permissions present (Task 14)
- **M10:** Document the single-Pixmap strategy explicitly in Task 6's comments

---

## Spec Coverage Gaps

| Spec Requirement | Status |
|---|---|
| Performance budgets (FPS, CPU, memory, startup) | ❌ No task measures these |
| i18n with Vietnamese translations | ❌ Dropdown exists but no i18n framework or translations |
| Backend-to-frontend error bridging | ❌ No notification wiring task |
| Tauri updater for auto-updates | ❌ Not configured |
| Windows MSI bundler | ❌ Task 15 only has NSIS |
| `cargo audit` + `cargo deny` in CI | ❌ Not in Task 14 |
| CSP enforcement verification | ⚠️ Config present but not verified |
| DPI-aware rendering | ⚠️ `dpi_scale` stored but not threaded through rendering |
| Auto-start implementation | ⚠️ Frontend toggle exists, no backend wiring |
| WCAG AA contrast verification | ❌ No verification step |

## Strengths

- Module decomposition (1 module = 1 task) is clean and traceable
- MockPlatformApi enables testing without real window systems
- ConfigStore write-verify-backup strategy prevents data loss
- Thread model is explicit: 4 named threads, mpsc + ArcSwap communication
- Dirty-rect + pre-allocated buffers in ZoneOverlay show performance awareness
- First-run onboarding (Task 13) is well-scoped with dismissible overlay + toast system
- Tauri deny-by-default capabilities correctly implemented
- Svelte auto-escape provides defense-in-depth against XSS
- Keyboard accessibility (arrow keys, tab, ARIA) covered in editor
