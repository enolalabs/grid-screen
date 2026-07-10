# Spec Review: Grid Screen Stitch UI Refresh

**Date:** 2026-07-10  
**Spec:** `docs/superpowers/specs/2026-07-10-stitch-ui-refresh-design.md`  
**Tech Stack:** Rust, TypeScript, Svelte 5, Vite, Tauri 2, Svelte stores, Tauri IPC; no database, cloud, or authentication  
**Reviewers:** Technical Quality, Performance, Security, Process & Operations, Product & UX

## Verdict: Needs Changes

The spec is well scoped and technically feasible for a local desktop UI refresh. It preserves the Rust backend and IPC boundary, has a coherent Stitch-derived visual system, and covers the main functional flows. Before implementation, several Important findings must be resolved so startup, status, editor interaction, security, packaging, and smaller-window behavior are not decided ad hoc during coding.

**Finding counts after consolidation:** 0 Critical, 12 Important, 6 Minor.

## Findings

### Technical Quality

#### Important

1. **Startup state machine is missing**
   - Section: `Error & System Status`; `Error handling and interaction states`
   - Issue: Initial `get_current_state`/settings failure and loading are not defined.
   - Impact: The shell may render misleading empty data or have no actionable recovery path.
   - Recommended fix: Define `loading`, `loaded`, and recoverable `initialization-failed` states with retry and a usable status affordance.

2. **Saved-layout badge source and store synchronization are ambiguous**
   - Section: `Saved Layouts`; `Workspace editor`; `Application Settings`
   - Issue: Default/active badges and post-mutation store updates have no single source of truth.
   - Impact: UI can become stale after save, delete, apply, or set-default actions.
   - Recommended fix: Reload or update `FrontendState` after mutations and explicitly derive default versus active identity.

3. **Font and asset delivery is incomplete**
   - Section: `Shared visual language`; `Design direction`; `Out of scope`
   - Issue: Fonts are required but remote fonts are prohibited; local packaging and fallback behavior are unspecified.
   - Impact: Offline visual fidelity can vary across machines.
   - Recommended fix: Define bundled local fonts or an approved fallback stack, and specify which approved local assets are runtime-consumed.

### Performance

#### Important

4. **Editor pointer interaction and IPC frequency are unspecified**
   - Section: `Workspace editor`
   - Issue: It is unclear whether `applyLayout` runs on every drag/resize event.
   - Impact: Pointer-event IPC can cause stutter and backend event backlog.
   - Recommended fix: Keep local draft state, update visual preview with `requestAnimationFrame`/bounded throttling, and persist/apply at drag end unless continuous apply is explicitly required.

5. **Editor rendering boundaries are unspecified**
   - Section: `Workspace editor`; `Shared visual language`
   - Issue: Grid, rulers, labels, inspector, and canvas rerender behavior is not defined.
   - Impact: Whole-shell rerenders may reduce drag smoothness.
   - Recommended fix: Isolate editor interaction state, keep static grid/rulers in CSS/SVG/canvas, and update only the affected zone/inspector data.

6. **No interaction performance acceptance target**
   - Section: `Testing and validation`; `Acceptance criteria`
   - Issue: Build and visual checks do not measure editor responsiveness or resource use.
   - Impact: Sluggish interaction could pass all listed tests.
   - Recommended fix: Add a representative multi-monitor/zone smoke check with bounded IPC calls, no visibly dropped-frame drag behavior, and an idle CPU baseline.

7. **Session notification history is unbounded**
   - Section: `Error & System Status`; `Error handling and interaction states`
   - Issue: Current-session history has no limit, deduplication, or cleanup policy.
   - Impact: Long sessions can grow state and DOM indefinitely.
   - Recommended fix: Retain a bounded recent history (for example, 100 entries), define ordering/deduplication, and clean listeners when views are destroyed.

### Security

#### Important

8. **Dynamic values need an explicit safe-rendering policy**
   - Section: `Workspace editor`; `Saved Layouts`; `Error & System Status`
   - Issue: Layout names and error messages may come from local config/backend data without a stated escaped-text rule.
   - Impact: Raw HTML rendering could allow WebView script execution or UI spoofing from tampered local data.
   - Recommended fix: Render dynamic values as escaped text only, prohibit raw HTML for dynamic data, and test malicious names/messages.

9. **IPC trust-boundary validation is underspecified**
   - Section: `Workspace editor`; `Application Settings`; `Acceptance criteria`
   - Issue: Frontend inputs and retained commands lack explicit Rust-side validation invariants.
   - Impact: Malformed geometry, strings, colors, or settings could corrupt config or trigger unsafe privileged behavior if backend validation regresses.
   - Recommended fix: Keep Rust as the trust boundary; validate finite numeric ranges, geometry, string lengths, color/language enums, and auto-start inputs independently of frontend validation; add negative tests.

10. **Downloaded Stitch artifacts need supply-chain handling**
    - Section: `Existing context`; `Design direction`
    - Issue: Reference HTML/screenshots are downloaded without specifying inspection, pinning, or exclusion from runtime build inputs.
    - Impact: Untrusted scripts/assets could enter tooling or the application bundle.
    - Recommended fix: Treat downloads as untrusted references, inspect and verify project/source, copy only approved static assets/styles, exclude HTML/scripts from build inputs, and verify no remote runtime references.

### Process & Operations

#### Important

11. **Packaged Tauri release validation is missing**
    - Section: `Testing and validation`; `Acceptance criteria`
    - Issue: A web build is required, but packaged install/launch and bundled resource checks are not.
    - Impact: Tauri resource paths, local fonts/assets, or IPC behavior may fail only in installed builds.
    - Recommended fix: Add a release smoke test for supported targets covering build, launch, first-run, local assets/fonts, and IPC-backed actions.

12. **Desktop rollback/recovery expectations are missing**
    - Section: `Testing and validation`; `Acceptance criteria`
    - Issue: Runtime recovery is defined, but recovery from a broken desktop release is not.
    - Impact: A bad refresh could strand users despite unchanged config schema.
    - Recommended fix: Retain the previous installer/build artifact, document downgrade expectations, and test representative existing config data.

### Product & UX

#### Important

The Product/UX review identified five additional implementation-shaping ambiguities:

- First-run versus returning-user empty state needs an explicit state matrix and independent completion semantics.
- System status must choose one interaction model: dedicated view, drawer, modal, or persistent panel.
- Accessibility needs concrete checks for keyboard order, focus restoration/trapping, names, live regions, reduced motion, and contrast target.
- Smaller desktop window behavior needs minimum dimensions, scrolling/overflow rules, toolbar behavior, and editor scaling.
- Startup loading/failure states must be defined for each view and shell status area.

## Minor Findings

- Document the supported platform/windowing matrix for this refresh.
- Add local diagnostic/copyable status summary or retainable local log access.
- Define memoization/versioning for saved-layout thumbnails.
- Define IPC mocking seams in frontend tests.
- Add a per-screen visual-fidelity checklist and permitted deviations.
- Clarify that six Stitch references consist of five user-facing experiences plus one internal Design System reference.

## Strengths

- Preserves the Rust/Tauri backend and existing IPC contracts.
- Uses a shared shell, tokens, primitives, and view-model layer instead of five independent page implementations.
- Keeps Stitch HTML/screenshots out of runtime and avoids CDN/network dependencies.
- Retains real layout editing, saving, applying, deleting, settings, and error behavior.
- Includes pending/error recovery, destructive confirmation, keyboard focus, accessibility intent, frontend/Rust tests, and manual visual comparison.
- Avoids unnecessary new backend health, telemetry, database, or cloud scope.

## Recommendation

Resolve the Important findings before creating the implementation plan. The most important sequence is: define the startup/first-run/status state model; define state synchronization and notification retention; define editor interaction/performance rules; add safe rendering and IPC validation invariants; then add packaged-release and desktop-size validation.
