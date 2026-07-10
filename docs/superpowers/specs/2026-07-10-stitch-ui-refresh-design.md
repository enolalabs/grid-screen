# Grid Screen Stitch UI Refresh — Design Spec

**Date:** 2026-07-10  
**Status:** Approved design; implementation pending  
**Scope:** Frontend visual refresh for the existing Tauri/Svelte application

## Goal

Integrate all six screens from Stitch project `3286229551374494803` into the existing Grid Screen application while preserving the existing Rust backend, Tauri IPC contracts, layout behavior, and monitor/window management semantics.

The implementation will prioritize close visual fidelity to the Stitch source and use English copy in the first pass. Six Stitch references are integrated: five user-facing experiences plus one internal Design System reference that is not a production route.

## Existing context

The current frontend has three Svelte routes (`LayoutEditor`, `LayoutManager`, and `Settings`) inside `App.svelte`, a simple horizontal tab bar, and a modal first-run guide. State is loaded through the existing IPC helpers and shared Svelte stores. The repository currently contains unrelated uncommitted Rust and documentation changes; the implementation must preserve them.

The Stitch project provides five desktop screens at 2560×2048 plus one Design System asset:

| Screen | Stitch ID | Application destination |
|---|---|---|
| Design System | `assets_a44c86b3972641e5be1ac75a7ded7975` | Internal tokens/components only |
| First-run / Empty State | `1c1a8638f3ab4a0893f58333e719fbf8` | Empty workspace/onboarding state |
| Editor - Workspace Management | `a8420839c84844e792e1485c24921af7` | Workspace route |
| Saved Layouts | `7537988f560e451ba0528ce03a5066a4` | Saved Layouts route |
| Application Settings | `8ab47bbdea804ba48fd4f5f2ab13f0c1` | Settings route |
| Error & System Status | `14117ae1c3134736a91ed2e5bd7887ff` | Status view/panel |

Hosted Stitch HTML and screenshot URLs are available from the Stitch project metadata and should be downloaded into a non-runtime reference directory during implementation. Runtime UI must not depend on remote URLs or CDNs.

Downloaded Stitch artifacts are untrusted reference material: verify the expected project and screen IDs, inspect before use, copy only approved static images or manually recreated styles, exclude downloaded HTML/scripts from build inputs, and verify the packaged runtime contains no remote references or unapproved assets.

## Design direction

Use one shared app shell and one token system, then reconstruct each screen as Svelte components connected to real application state. Do not embed the Stitch HTML as standalone pages and do not use screenshots as runtime backgrounds.

### Shell and navigation

- Replace the current horizontal tab bar with a fixed left sidebar of approximately 280px.
- The sidebar contains Grid Screen branding, navigation, and runtime status.
- User-facing navigation contains `Workspace`, `Saved Layouts`, and `Settings`.
- The sidebar runtime-status affordance opens a dedicated `System Status` view. This view is not part of the three primary navigation items, but has a stable view state, a `Back to Workspace` action, and preserves current-session notification history while the shell remains mounted.
- The main content area is fluid, uses the Stitch graphite canvas, and follows a 4px spacing grid.
- The supported minimum window size for this refresh is 1024×720. Below the Stitch reference viewport, the sidebar remains fixed, main content scrolls, toolbars wrap rather than clip, and the editor canvas can scroll horizontally and vertically.
- Keep view switching as frontend state rather than introducing URL routing, avoiding changes to the Tauri window flow.
- `Design System` is not listed in navigation.
- The application opens on `Workspace` after first-run completion.
- The status area exposes snapping active/paused state, monitor count, and a path to system status.

### Shared visual language

Create a single theme stylesheet and shared primitives. The initial English UI must match Stitch wording and hierarchy; existing i18n infrastructure remains available for a later Vietnamese pass.

Core tokens:

- Canvas: `#0F0D15` and `#15121B`
- Surfaces: `#1D1A23`, `#211E27`, `#2C2832`, `#37333D`
- Borders: `#494454`
- Primary violet: `#8B5CF6`; focus/bright primary: `#D0BCFF`
- Main text: `#E7E0ED`; secondary text: `#CBC3D7`
- Secondary slate: `#64748B`
- Warning/tertiary: `#FFB869`
- Error: `#FFB4AB`
- UI typeface: Geist
- Technical labels/data: JetBrains Mono
- Standard control radius: 4px
- Panel radius: 8px
- Grid/ruler intersections: 0px radius
- Spacing units: multiples of 4px; page margin 24px; standard panel padding 16px; compact grouping 8px

Geist and JetBrains Mono are bundled as local static font assets and loaded with `@font-face`; the runtime must not fetch fonts from a network. CSS defines a system sans-serif fallback for Geist and a system monospace fallback for JetBrains Mono. Packaged-app smoke tests verify that local font assets resolve in the installed build.

Depth is expressed through tonal surfaces and one-pixel outlines instead of large shadows. Focus and selected states use a violet border or subtle inner glow. Window zones use dashed borders when inactive and solid violet borders with a low-opacity violet fill when active.

## Screen behavior and data flow

### First-run / Empty State

Replace the current onboarding modal with an in-shell state selected from this explicit matrix:

| Condition | State | Behavior |
|---|---|---|
| `settings.first_run_completed === false` | First-run onboarding | Show setup guidance and `Create your first layout`; navigation remains available. The user can enter Workspace, and onboarding is completed only through the existing completion action, which persists `first_run_completed = true`. |
| `settings.first_run_completed === true` and no usable saved/active layout | Empty workspace | Show the normal empty-workspace treatment and `Create your first layout`; do not modify `first_run_completed`. |
| Saved layouts exist but none is usable for the connected monitors | Layout recovery | Explain the monitor/layout mismatch and provide actions to open Workspace or Saved Layouts; do not silently overwrite or mark first-run state complete. |

A layout is usable when it contains at least one zone and its `monitor_id` matches an available monitor. A saved layout can be usable even when it is not the current active layout. The primary action switches to Workspace; it does not persist settings by itself. All three states keep the shell navigation available and use the Stitch visual language with state-specific copy.

### Workspace editor

Retain all current editor behavior:

- Read monitors and active layouts from `currentState`.
- Create, rename, move, resize, keyboard-adjust, and delete zones.
- Preserve 12-column snapping and fractional coordinates.
- Continue using `applyLayout` and `saveLayout` for live apply and persistence.

Zone create/move/resize interactions use local draft state. Pointer movement updates the affected zone and visual preview synchronously, with grid/ruler repaint work coalesced to `requestAnimationFrame` where needed. The editor does not call Tauri IPC for every pointer event: `applyLayout` runs only from the explicit `Apply Live` action, and `saveLayout` runs only from the explicit Save action. The draft remains available after a failed apply/save so the user can retry or continue editing.

Editor interaction state is isolated to the canvas and selected-zone inspector. Static grid/ruler visuals use CSS or a single lightweight SVG/canvas layer; unchanged zone labels, sidebar, toolbar, and settings panels are not rebuilt for every pointer movement.

Rebuild the presentation around the Stitch workspace screen: a large monitor canvas, technical toolbar, grid/rulers, zone labels, coordinate/dimension data, and a contextual inspector or metadata panel. The editor remains functional when monitors are missing and must show the Stitch empty/error treatment rather than throwing.

### Saved Layouts

Use the existing `listLayouts`, `setDefaultLayout`, and `deleteLayout` IPC functions. Present layouts in the Stitch card/list treatment with:

- layout name and technical metadata;
- a miniature zone thumbnail generated from saved zone coordinates;
- zone count and monitor/arrangement information;
- default/active state badge;
- set-default and delete actions;
- an empty state linking back to Workspace.

The frontend view model includes the existing backend `settings.default_layout_id` field. The default badge is derived from that ID; the active badge is derived from whether the saved layout's monitor has a matching entry in `FrontendState.active_layouts`. After any saved-layout mutation, reload `getCurrentState` and replace the relevant shared stores so default/active badges cannot become stale.

Destructive actions require confirmation and preserve the existing backend operation. IPC failures appear through the shared error/status mechanism.

### Application Settings

Keep the existing settings contract and fields:

- auto-start;
- default gap;
- default margin;
- accent color;
- language;
- first-run completion state.

Group settings according to the Stitch layout, use the shared controls, and provide clear saved/failed feedback. The initial refreshed copy is English. The existing language selector remains functional.

### Error & System Status

Add a dedicated frontend `System Status` view reachable from the sidebar runtime-status affordance. It consumes the existing notifications and `FrontendState` data and presents:

- snapping active/paused;
- monitor availability/count;
- configuration load/save feedback;
- recent error or warning messages;
- safe fallback labels such as `Unknown` or `No recent errors` where the backend has no richer health data.

The view has a clear title, a `Back to Workspace` action, keyboard-focusable status rows, and remains directly reachable from the sidebar. It is not a modal or drawer, so it does not obscure Workspace content or require overlay focus management.

Session notification history is a bounded store containing the latest 100 notifications, ordered newest-first. Consecutive identical messages at the same severity are coalesced. The history survives navigation while the shell is mounted, supports Clear history, and is reset on application restart. Error text is displayed as received from the backend but does not expose configuration contents or sensitive local paths when a safer summary is available.

### Startup and loading states

The shell renders immediately and owns an initialization state machine:

- `loading`: sidebar and status affordance render, while each data-dependent view shows a Stitch-style skeleton/loading treatment;
- `loaded`: the normal state matrix and route content render from `FrontendState`;
- `initialization-failed`: the shell remains navigable, the status area reports `Initialization failed`, and a `Retry` action repeats the existing state-loading request.

An initialization failure never masquerades as a loaded empty state. View-level list/settings requests use the same loading, loaded-empty, and failed-with-retry distinction. Previously loaded data remains visible during a later refresh failure.

Do not invent a new backend health protocol. Existing Tauri events (`user-notification`) remain the source of asynchronous notifications. The view must never block access to Workspace or Settings after a recoverable error.

## Component boundaries

The implementation should establish these focused frontend boundaries:

- `src/lib/theme.css`: design tokens, typography, global focus and surface rules.
- `src/lib/components/AppShell.svelte`: fixed sidebar plus main content frame.
- `src/lib/components/Sidebar.svelte`: navigation and runtime status.
- `src/lib/components/TopBar.svelte`: view title, actions, and contextual status.
- `src/lib/components/Panel.svelte`: common bordered tonal container.
- `src/lib/components/Button.svelte`, `Badge.svelte`, and form primitives: shared controls.
- `src/lib/components/EmptyState.svelte`, `StatusRow.svelte`, and `ErrorPanel.svelte`: reusable state treatments.
- `src/lib/icons.ts`: local inline SVG/icon definitions; no remote runtime assets.
- `src/lib/view-models.ts`: presentation mapping from IPC/store types to stable UI data.
- Existing route files: screen-specific composition and interactions only.

Avoid a broad unrelated refactor. Components should communicate through explicit props/events and keep IPC calls in route-level handlers or the existing `ipc.ts` boundary.

Dynamic values such as layout names, monitor names, technical metadata, and notification messages are rendered as escaped text only. Runtime UI must not use raw HTML rendering for backend- or user-controlled values.

The Rust IPC boundary remains authoritative for validation. Frontend checks are UX-only. Existing and retained commands must independently validate finite numeric values and geometry ranges, bounded string lengths, valid accent-color and language values, and safe auto-start inputs. Malformed-input regression tests belong with the existing Rust IPC/config tests.

## Error handling and interaction states

- All async actions expose pending/disabled state and restore controls after success or failure.
- IPC errors are normalized into the shared notification helper and remain visible in the status panel/history for the current session.
- Existing loaded state remains usable after a failed save, delete, apply, or settings operation.
- First-run persistence failures keep the empty state visible and show an actionable error.
- Missing monitors, empty layouts, corrupted config notifications, and paused snapping are intentional visual states, not exceptional rendering paths.
- Keyboard focus, button labels, dialog roles, and contrast must remain accessible in the dark theme.
- Accessibility acceptance checks require every interactive element to be keyboard reachable with a visible focus indicator; logical sidebar-to-content tab order; accessible names for icon-only controls; focus trapping and restoration for confirmation dialogs; appropriate live-region behavior for status changes; reduced-motion-safe transitions; and WCAG AA contrast for text and controls.

## Testing and validation

Frontend validation must include:

1. TypeScript/Svelte production build.
2. Existing `LayoutEditor` tests updated for the new shell/component structure.
3. Tests for first-run/empty rendering, navigation selection, saved-layout actions, settings save feedback, and status/error rendering.
4. Keyboard focus and dialog behavior checks for destructive confirmation and primary actions.
5. Rust test suite to confirm frontend-only changes did not affect backend behavior.
6. Manual visual comparison at the Stitch desktop viewport dimensions, checking sidebar width, panel alignment, typography, token colors, spacing, and editor proportions.
10. Repeat visual validation at the 1024×720 minimum window size, checking that navigation, settings, status, and editor actions remain reachable without clipping.
7. Packaged Tauri release smoke test for each supported target: build, launch in a clean environment, verify local fonts/assets, complete first-run, retry initialization failure, and exercise at least one IPC-backed action.
8. Regression test with representative existing configuration data, plus a documented previous-build artifact/downgrade path if a packaged UI release must be rolled back.
9. Editor interaction baseline for the expected maximum monitor/zone configuration: no visibly dropped-frame drag behavior, no IPC call per pointer event, and no sustained high idle CPU caused by the refreshed UI.

After `applyLayout`, `saveLayout`, `deleteLayout`, `setDefaultLayout`, or settings save succeeds, the route refreshes the relevant shared store from the existing IPC source of truth. Failed mutations leave the current draft/loaded state intact. Frontend tests mock the `ipc.ts` boundary and Tauri notification listener to cover loading, success, failure, pending, and retry paths.

## Out of scope

- Changes to Rust layout, monitor, drag detection, overlay, tray, or config schemas.
- New backend health/telemetry IPC commands.
- Wayland support.
- Making Design System a production navigation route.
- Runtime dependency on Stitch-hosted URLs, external fonts, or CDNs.
- Vietnamese copy rewrite in this first visual-fidelity pass.

## Acceptance criteria

- All three existing functional areas are reachable through the new Stitch-style shell.
- First-run and empty states are rendered as deliberate screens, not the old generic modal.
- Workspace editing, saving, applying, deleting, default selection, and settings persistence still call the existing IPC paths.
- Error/status conditions are visible and recoverable without losing loaded state.
- The visual system is centralized and the five user-facing Stitch screens share the same tokens and primitives.
- Build and relevant tests pass, with no changes required to Rust backend contracts.
