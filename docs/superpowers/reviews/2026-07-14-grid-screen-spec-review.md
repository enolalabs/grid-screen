# Spec Review: Grid Screen MVP Design

**Date:** 2026-07-14
**Spec:** `docs/superpowers/specs/2026-07-14-grid-screen-mvp-design.md`
**Tech Stack:** Rust, Svelte/TypeScript, Tauri 2, Linux X11
**Reviewers:** rust-engineer, typescript-pro, performance-engineer, security-auditor, devops-engineer, product-manager

---

## Verdict: Needs Changes

**Summary:** Architecture is sound and well-scoped, but the spec is missing critical implementation contracts — type definitions, Tauri command schema, component decomposition, distribution pipeline, and logging. Several UX interaction rules (zone replacement, layout edit semantics, Snap placeholder UI) are ambiguous or deceptive.

**Finding counts:** 14 Critical, 32 Important, 26 Minor

---

## Critical (Must Fix)

### Architecture & Type Contracts

1. **No Tauri command/event type signatures (Tech §7.1, §8.1)**
   - The spec declares "typed Tauri commands" but defines zero command signatures, event payloads, or shared TypeScript/Rust types. This is the primary interface between Rust and Svelte.
   - **Fix:** Add a command/event schema section listing: `get_screens()`, `get_layouts()`, `get_window_catalog()`, `arrange_windows()`, `save_layout()`, `get_settings()`, `update_settings()`, plus event types (`workspace_changed`, `screen_changed`).

2. **No Svelte component tree or store architecture (Frontend §5, §7.1)**
   - The spec names views but identifies zero Svelte components or stores. The mockup uses a monolithic `state` object — this must not carry into implementation.
   - **Fix:** Define component tree (ArrangeView, WindowCatalog, CanvasArea, ZoneSlot, DetailPanel, LayoutsView, SettingsView) and Svelte writable/derived stores (assignments, layout, screen, settings, windows, systemStatus, toasts).

3. **No window decoration offset strategy (Tech §6.3, §14.AC2)**
   - AC2 says windows must land within target zones "accounting for decoration differences," but there is no mechanism for computing `_NET_FRAME_EXTENTS` or corrective offsets in the PlatformAdapter or Layout Engine.
   - **Fix:** Add `get_window_frame_extents(id) -> Rect` to PlatformAdapter. The Arrange Orchestrator applies the offset before `move_resize_window`.

4. **Layout properties vs. session overrides ambiguity (Tech §5.3, §5.4, §6.1; Product C1)**
   - Sliders in Arrange mutate Ratio/Gap/Margin, but it's unclear whether changes are permanent (mutate saved layout) or session-only (reset on layout switch).
   - **Fix:** Split model into `LayoutDefinition` (immutable reference) and `ArrangementContext` (ephemeral override initialized from definition). Sliders mutate context only. "Duplicate" or explicit "Save" persist to new/existing LayoutDefinition.

5. **Drag-and-drop mechanism not specified (Frontend §5.3)**
   - HTML5 DnD API has known reliability issues in Tauri 2's Linux WebKitGTK webview. The spec doesn't verify compatibility or specify a fallback (pointer-based drag).
   - **Fix:** Specify pointer-event-based drag (mousedown → mousemove → mouseup with custom ghost element) as the primary mechanism. Document reassignment and drop-outside-zones behavior.

### UX Specification Gaps

6. **Canvas preview must match actual screen aspect ratio (Product §5.3)**
   - The mockup uses a fixed 16:10 aspect ratio. Real monitors are 16:9, 21:9, 4:3 — the preview must match the selected screen's resolution.
   - **Fix:** Require canvas aspect ratio to match selected screen geometry.

7. **Zone replacement semantics unspecified (Product §5.3)**
   - Dropping a window onto an occupied zone — does it replace or reject?
   - **Fix:** Document that dropping on an occupied zone replaces the previous occupant (returns to unassigned catalog).

8. **Non-functional Snap UI is deceptive (Product §5.3, §5.5)**
   - Snap toggles appear interactive (animate on click) but do nothing in MVP. Users will lose trust.
   - **Fix:** Render Snap controls as disabled/greyed-out with tooltip: "Snap coming in a future update."

9. **"Edit" on saved layout has no defined behavior (Product §5.4)**
   - Layouts view shows "Edit" action but all adjustment happens in Arrange.
   - **Fix:** "Edit" loads the saved layout into Arrange with its saved ratio/gap/margin values. Adjust sliders, changes save back on Arrange success or explicit save.

10. **"Duplicate" on built-in preset ambiguous (Product §5.4)**
    - Does it immediately create a copy or load into Arrange for tweaking?
    - **Fix:** "Duplicate" opens an inline name prompt, creates a saved copy immediately, navigates to Arrange with it selected.

### Delivery & Operations

11. **No binary distribution pipeline (DevOps §4.1, §7.1)**
    - No mention of how the Tauri binary is built, packaged (AppImage/deb), signed, or distributed.
    - **Fix:** Add Build & Distribution section: `cargo tauri build --bundles deb,appimage`, GitHub Releases, Tauri updater plugin.

12. **No versioning or update mechanism (DevOps §7.1)**
    - Users on old versions have no way to discover or install updates.
    - **Fix:** Semantic versioning + Tauri updater plugin + `version.json` on GitHub Releases.

13. **No logging or diagnostic output (DevOps §9, §13.4)**
    - When users report bugs, there is no mechanism to understand what happened.
    - **Fix:** Add file-based logging (tracing crate) with rotation to `~/.config/grid-screen/logs/`. Add "Copy diagnostics" button in System Status.

14. **No CI/CD pipeline (DevOps §13)**
    - No automated build, test, or release workflow.
    - **Fix:** Add GitHub Actions: PR checks (cargo test/clippy, npm lint/check), release build on tag, nightly full test suite.

---

## Important (Should Fix)

_Consolidated from 32 findings across all reviewers. Key highlights below; full details in review subsections._

**Data Flow & State:**
- Workspace-change during arrangement not defined (Tech I1)
- Single-instance behavior mechanism unspecified (Tech I2, Sec I6)
- Window catalog refresh lifecycle missing (Tech I3)
- Slider debouncing needed for geometry recomputation (Perf I4)
- Settings "derived from last used" data flow unclear (Frontend 10)
- Window re-open state restoration unspecified (Product I4)

**X11 & Events:**
- Event subscription mechanism must be blocking, not polling (Perf I1)
- Double window enumeration during arrange flow (Perf I3)
- Event stream types undefined (Tech I6)
- Frontend event subscription and cleanup not specified (Frontend 5, 11)

**Privacy & Security:**
- Window titles in toast/error messages leak sensitive info (Sec I1)
- Config file permissions not specified — must be 0600/0700 (Sec I2)
- Backup files multiply persistence of accidentally pasted secrets (Sec I3)
- Autostart .desktop file permissions (Sec I5)

**UX Details:**
- Action bar info should be "N of Z zones filled" not "N of M windows assigned" (Product I2)
- Default Gap/Margin derivation rule ambiguous (Product I3)
- Minimize to Tray toggle when tray unavailable — should be disabled (Product I8)
- Toast messages too vague ("Done!" vs "Arranged 3 windows on DP-1") (Product M1)
- Window Catalog empty state conflates "no match" and "no windows" (Product M2)

**Layout & Zone:**
- Layout creation flow UI not described (Product M5)
- Focus + Stack zone ordering not in prose (Product M4)
- 3 Columns and 3 Wide Center share identical quick-layout icon (Product I7)
- Minimum zone size enforcement for very small screens (Perf M4)
- Layout name validation constraints missing (Tech I5)
- CSS grid rendering complexity for varying layout shapes (Frontend 6)

**Performance & Resource:**
- No perf measurement methodology or benchmark targets (Perf I2)
- No performance regression testing gates (Perf I6)
- Config read-back validation adds unnecessary I/O (Perf M1)
- Event listeners not cleaned up when webview destroyed (Perf M2)

**Operations:**
- Config migration path for schema version changes (DevOps I1)
- Autostart integration specifics (XDG autostart spec) (DevOps I2)
- Tray detection logic for unsupported WMs (DevOps I3)
- No `--diagnose` CLI flag for environment verification (DevOps I4)

**Implementation Details:**
- Tray Open Grid Screen window re-creation logic (Tech I7)
- "Arrange in progress" UI state (loading, disable tabs) (Frontend 7)
- Toast system with queue instead of single-element (Frontend 8)
- App icon color derivation should be in Rust, not hardcoded (Frontend 12)
- Window title HTML injection via drop handler (Sec I8)

---

## Strengths

Across all 5 reviewers, these themes emerged:

1. **Architecture is well-chosen.** Rust core + Svelte webview via typed Tauri IPC, with PlatformAdapter trait for X11 isolation and MockPlatformAdapter for testing — this is the correct pattern for this class of app.

2. **Failure handling is thorough.** Section 9's condition-behavior table covers real X11 failure modes (WM rejection, decoration enforcement, screen disconnect, workspace change) with concrete behaviors — unusually disciplined for an MVP spec.

3. **Safety invariants are explicit and testable.** "Unassigned windows never changed," "validate all before changing any," "opaque session-only window IDs," "no continuous polling while idle" — all falsifiable and QA-able.

4. **Security posture is strong for the threat model.** No network, no auth, explicit CSP, XSS prevention by design (titles rendered as text), atomic config writes with backup rotation, narrow webview command allowlist.

5. **Scope discipline is excellent.** The non-goals section clearly walls off Wayland, modifier snap, keyboard accessibility, workspace switching, and localization — protecting against "one more thing" creep during implementation.

6. **Performance awareness at design stage.** Quantified targets (idle <1%, arrange <1s, no polling) with an 8-hour soak test for resource leaks — this is rare in early specs and well-calibrated for a desktop tool.

---

## Recommendations

1. **Write the shared type crate first.** All critical gaps trace back to missing type definitions. A `shared-types/` crate with Rust structs + TypeScript interfaces is the single highest-leverage addition.

2. **Spike the drag-and-drop on Tauri 2 WebKitGTK immediately.** This is the highest-risk interaction. Verify pointer-based drag works before committing to the HTML5 DnD API.

3. **Add logging from day one.** `tracing` + file appender is an afternoon of work and pays for itself on the first bug report.

4. **GitHub Actions for CI is cheap and fast.** 50 lines of YAML prevents regressions and enables automated release builds. Add it during implementation setup.

## Reviewer Details

### Technical (rust-engineer + typescript-pro)
- 3 Critical, 7 Important, 5 Minor (rust-engineer)
- 3 Critical, 5 Important, 5 Minor (typescript-pro)
- Combined: 3 Critical, 10 Important, 8 Minor

### Performance (performance-engineer)
- 0 Critical, 6 Important, 5 Minor

### Security (security-auditor)
- 0 Critical, 3 Important, 6 Minor

### Process (devops-engineer)
- 4 Critical, 4 Important, 5 Minor

### Product (product-manager)
- 4 Critical, 8 Important, 6 Minor
