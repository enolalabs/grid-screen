# Plan Review: Grid Screen Stitch UI Refresh

**Date:** 2026-07-10  
**Plan:** `docs/superpowers/plans/2026-07-10-stitch-ui-refresh-implementation.md`  
**Spec:** `docs/superpowers/specs/2026-07-10-stitch-ui-refresh-design.md`  
**Tech Stack:** Rust, TypeScript, Svelte 5, Vite, Tauri 2, Vitest, Testing Library, Svelte stores, Tauri IPC  
**Reviewers:** Technical Quality, Performance, Security, Process & Operations, Product & UX

## Verdict: Needs Changes

The plan has strong task ordering, preserves the backend boundary, and covers nearly all user-facing requirements. It is not execution-ready yet because two interface/test-contract problems can directly cause inconsistent implementation, and several important validation steps remain too vague.

**Finding counts after consolidation:** 2 Critical, 11 Important, 6 Minor.

## Critical Findings

1. **Thumbnail memoization interface is inconsistent**
   - Reference: `Task 5, Interfaces` and `Task 5, Step 2`
   - Issue: `LayoutThumbnail.svelte` accepts only `zones` and `label`, but memoization requires layout ID plus zone signature.
   - Fix: Add `layoutId: string` to the component props and use it with a stable zone signature for memoization.

2. **Test steps are not executable test contracts**
   - Reference: `Task 2, Step 1`; `Task 3, Step 1`; `Task 4, Step 1`; `Task 5, Step 1`; `Task 6, Step 1`; `Task 7, Steps 1–2`
   - Issue: Steps say “test that” or “add assertions” without fixture shapes, exact mocks, event setup, and expected assertions.
   - Fix: Add concrete test contracts and representative code snippets for each suite, including IPC mock returns/rejections, Svelte event setup, and exact pending/failure/retry/store assertions.

## Important Findings

1. **First-run completion action is not defined** — `Task 4, Step 4` refers to an existing action even though the modal is being replaced. Define the in-shell control and transition.
2. **Workspace callback types are incomplete** — `Task 4, Interfaces` lists untyped callbacks. Define exact signatures and draft-zone ownership.
3. **Toast state and persistent history are conflated** — `Task 3`/`Task 6` need separate `toastNotifications` and `notificationHistory` stores/selectors.
4. **Workspace success refresh is missing** — `Task 4` must refresh `currentState`/stores after successful Apply Live and Save, while preserving draft after failure.
5. **Rust-side IPC validation has no implementation task** — `Task 7` must inspect/add validation and negative tests for geometry, strings, colors, language, and auto-start.
6. **Editor performance baseline is not executable** — add a defined representative monitor/zone workload, measurement procedure, and recorded result for no per-pointer IPC, no visible drag stutter, and idle CPU.
7. **`requestAnimationFrame` requirement is too implicit** — `Task 4` must explicitly schedule dynamic repaint once per frame while keeping unchanged panels out of the update path.
8. **Font fallback weakens the approved spec** — `Task 1` must require approved local font assets; no silent fallback substitution unless the spec is revised.
9. **Packaged bundle inspection is missing** — add a built-output scan for remote URLs, reference HTML/scripts, and unapproved assets.
10. **Rollback is only nominal** — record previous artifact/version, verify it launches with representative config, and document downgrade limitations.
11. **Pending/recovery tests are incomplete** — add explicit disabled/one-call-only/re-enabled assertions for Workspace Apply/Save and Settings Save.

## Minor Findings

- Verify Stitch metadata/project/screen IDs and record hashes/content types before download.
- Use explicit file lists plus `git diff --cached` checks; broad directory staging may capture unrelated user changes.
- Name supported OS/windowing targets in packaged validation.
- Add a screen-specific visual checklist with screenshots/pass-fail notes at 2560×2048 and 1024×720.
- Test CSS `prefers-reduced-motion` behavior rather than an arbitrary reduced-motion class.
- List required font weights explicitly.

## Strengths

- Tasks are ordered from references and primitives to shell, routes, and validation.
- The plan correctly keeps Stitch artifacts out of runtime and preserves Tauri IPC/backend contracts.
- First-run, empty, recovery, loading, retry, bounded history, local draft state, badges, thumbnails, CSP, accessibility, packaged smoke tests, and rollback intent are all represented.
- No unnecessary backend health protocol, database, cloud, auth, or CI/CD scope was added.

## Recommendation

Resolve the two Critical findings first, then make the Important interfaces and validation procedures concrete. After those changes, the plan should be ready for execution.
