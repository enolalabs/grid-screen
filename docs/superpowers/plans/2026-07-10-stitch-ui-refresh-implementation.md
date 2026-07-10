# Grid Screen Stitch UI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current Svelte tab-bar UI with a Stitch-faithful Grid Screen shell and five connected user-facing experiences while preserving all existing Rust/Tauri IPC and layout behavior.

**Architecture:** Keep `App.svelte` as the frontend state coordinator and retain the existing `ipc.ts` boundary. Add a shared theme, small presentational components, a bounded session notification store, and explicit loading/first-run/status view models; rebuild the three existing routes around those primitives and add the dedicated System Status view. Stitch HTML/screenshots remain untrusted, non-runtime references; only approved local fonts/images and manually recreated styles enter the build.

**Tech Stack:** Tauri 2.x, Rust backend, Svelte 5 runes, TypeScript, Vite, Vitest, Testing Library, existing Svelte stores and Tauri IPC.

## Global Constraints

- Preserve Rust backend behavior, Tauri IPC command names/signatures, config schema, monitor/window management, snapping, tray, and overlay behavior.
- Use six Stitch references as source material: five user-facing experiences plus one internal Design System reference.
- Use English copy in this pass; keep existing i18n infrastructure intact.
- Use local bundled Geist and JetBrains Mono fonts with system fallbacks; no CDN, remote font, remote image, or runtime Stitch URL.
- Use `#0F0D15`/`#15121B` graphite canvas, `#1D1A23`/`#211E27`/`#2C2832`/`#37333D` surfaces, `#8B5CF6` primary, 4px spacing, 4px controls, 8px panels.
- Keep the minimum supported window size at 1024×720; preserve the fixed sidebar and allow main/editor scrolling below the reference viewport.
- Render backend/user-controlled values as escaped text only; never use raw HTML for dynamic content.
- Treat Rust IPC as the trust boundary; frontend validation is UX-only.
- Editor pointer interactions use local draft state; `applyLayout` runs only from `Apply Live`, `saveLayout` only from Save.
- Keep the working tree’s unrelated user changes; never reset or overwrite them.

---

### Task 1: Capture Stitch references and establish local asset policy

**Files:**
- Create: `docs/superpowers/references/stitch-grid-screen/README.md`
- Create: `docs/superpowers/references/stitch-grid-screen/design-system.md`
- Create: `src/assets/fonts/Geist-Regular.woff2`
- Create: `src/assets/fonts/Geist-SemiBold.woff2`
- Create: `src/assets/fonts/JetBrainsMono-Regular.woff2`
- Create: `src/assets/fonts/JetBrainsMono-Medium.woff2`
- Create: `src/assets/stitch/*` only for approved static images/icons used by the final UI
- Modify: `.gitignore` only if reference downloads need an explicit non-runtime directory rule

**Interfaces:**
- Produces a local reference record containing the project ID, six source IDs, source titles, dimensions, and the exact hosted screenshot/HTML URLs used for inspection.
- Produces the approved local Geist and JetBrains Mono font files required by the spec.
- Does not produce runtime HTML or execute downloaded scripts.

- [ ] **Step 1: Create the reference manifest**

Retrieve metadata with Stitch `get_project` and `list_screens`, then assert project `3286229551374494803`, all six supplied source IDs, five screen titles, and five 2560×2048 dimensions before downloading. If metadata cannot be retrieved or an ID/title/dimension differs, stop Task 1. Record the verified values and design tokens in `docs/superpowers/references/stitch-grid-screen/README.md`; state that downloaded HTML is reference-only and excluded from Vite/Tauri inputs.

- [ ] **Step 2: Download hosted HTML and screenshots with `curl -L`**

Copy the exact `downloadUrl` values returned by verified Stitch metadata into the reference manifest, then run one `curl -L` command for every manifest HTML/screenshot entry. Create `html/` and `screenshots/` subdirectories first. Verify HTML responses are `text/html`, screenshots decode as images, and record `sha256sum` values in the manifest. Repeat for first-run, editor, saved layouts, settings, and error/status. Inspect files as untrusted reference material; do not import downloaded HTML into `src/`.

- [ ] **Step 3: Write the extracted internal design-system notes**

Copy the approved token values and component observations into `design-system.md`: surfaces, borders, typography, radii, spacing, zone treatment, sidebar width, and ruler/grid treatment. Do not copy executable HTML or remote URLs into runtime source.

- [ ] **Step 4: Add local font assets and verify fallback behavior**

Add approved, licensed local `.woff2` assets under `src/assets/fonts/`: Geist 400 and 600; JetBrains Mono 400 and 500. System stacks remain CSS fallbacks for load failure only, not an acceptable substitute for missing bundled assets. If approved font files cannot be included, stop Task 1 and request a spec decision before implementation; never fetch fonts at runtime.

- [ ] **Step 5: Verify the reference directory is not runtime input**

Run:

```bash
rg -n "usercontent|lh3.googleusercontent|http://|https://" src src-tauri || true
```

Expected: no Stitch-hosted URL is required by runtime source. Keep reference URLs confined to `docs/superpowers/references/stitch-grid-screen/`.

- [ ] **Step 6: Commit the reference-only changes**

```bash
git add docs/superpowers/references/stitch-grid-screen/README.md docs/superpowers/references/stitch-grid-screen/design-system.md docs/superpowers/references/stitch-grid-screen/html docs/superpowers/references/stitch-grid-screen/screenshots src/assets/fonts/Geist-Regular.woff2 src/assets/fonts/Geist-SemiBold.woff2 src/assets/fonts/JetBrainsMono-Regular.woff2 src/assets/fonts/JetBrainsMono-Medium.woff2
git diff --cached --check
git commit -m "docs: capture Stitch UI references"
```

---

### Task 2: Add theme tokens, font loading, icons, and shared primitives

**Files:**
- Create: `src/lib/theme.css`
- Create: `src/lib/icons.ts`
- Create: `src/lib/components/Button.svelte`
- Create: `src/lib/components/Badge.svelte`
- Create: `src/lib/components/Panel.svelte`
- Create: `src/lib/components/StatusRow.svelte`
- Create: `src/lib/components/EmptyState.svelte`
- Create: `src/lib/components/ErrorPanel.svelte`
- Create: `src/lib/components/__tests__/primitives.test.ts`
- Modify: `src/main.ts`

**Interfaces:**
- `Button.svelte`: props `{ variant: "primary" | "ghost" | "danger"; type?: "button" | "submit"; disabled?: boolean; loading?: boolean; ariaLabel?: string }` and forwards click/slot content.
- `Badge.svelte`: props `{ tone: "primary" | "success" | "warning" | "error" | "muted"; text: string }`.
- `Panel.svelte`: props `{ title?: string; eyebrow?: string; interactive?: boolean }` and renders a named header plus default slot.
- `EmptyState.svelte`: props `{ eyebrow: string; title: string; description: string; actionLabel?: string; completionLabel?: string; onboarding?: boolean; onAction?: () => void; onCompleteOnboarding?: () => void }`.
- `ErrorPanel.svelte`: props `{ title: string; message: string; retry?: () => void }`.
- `StatusRow.svelte`: props `{ label: string; value: string; tone?: "primary" | "success" | "warning" | "error" | "muted" }`.

- [ ] **Step 1: Write primitive tests**

Test that primary/ghost/danger buttons expose accessible names, disabled/loading states prevent action, badges render tone classes, EmptyState invokes separate primary and completion callbacks, and ErrorPanel renders Retry only when supplied. Include this exact separation test:

```ts
it("keeps navigation separate from onboarding completion", async () => {
  const onAction = vi.fn();
  const onCompleteOnboarding = vi.fn();
  const view = render(EmptyState, { props: {
    eyebrow: "FIRST RUN", title: "Build your workspace", description: "Create zones",
    actionLabel: "Create your first layout", completionLabel: "Finish setup", onboarding: true,
    onAction, onCompleteOnboarding,
  }});
  await fireEvent.click(view.getByRole("button", { name: "Create your first layout" }));
  expect(onAction).toHaveBeenCalledOnce();
  expect(onCompleteOnboarding).not.toHaveBeenCalled();
  await fireEvent.click(view.getByRole("button", { name: "Finish setup" }));
  expect(onCompleteOnboarding).toHaveBeenCalledOnce();
});
```

- [ ] **Step 2: Implement `theme.css`**

Define CSS variables for the exact Stitch palette, typography, radii, spacing, focus ring, surface levels, and scrollbar treatment. Add local `@font-face` rules for Geist weights 400/600 and JetBrains Mono weights 400/500, plus load-failure fallbacks:

```css
@font-face { font-family: "Geist"; src: url("../assets/fonts/Geist-Regular.woff2") format("woff2"); font-weight: 400; font-display: swap; }
@font-face { font-family: "JetBrains Mono"; src: url("../assets/fonts/JetBrainsMono-Regular.woff2") format("woff2"); font-weight: 400; font-display: swap; }
:root { --canvas: #0f0d15; --surface-1: #1d1a23; --surface-2: #211e27; --surface-3: #2c2832; --surface-4: #37333d; --border: #494454; --primary: #8b5cf6; --primary-bright: #d0bcff; --text: #e7e0ed; --text-muted: #cbc3d7; --mono: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace; --sans: "Geist", ui-sans-serif, system-ui, sans-serif; }
```

- [ ] **Step 3: Implement primitives with escaped slots and focus behavior**

Use normal Svelte text interpolation for all dynamic values. Add `:focus-visible` outlines, 32px control height, 4px control radius, and reduced-motion-safe transitions.

- [ ] **Step 4: Import theme globally and run primitive tests**

Import `./lib/theme.css` from `src/main.ts`, then run:

```bash
npx vitest run src/lib/components/__tests__/primitives.test.ts
```

Expected: all primitive tests pass.

- [ ] **Step 5: Commit shared visual foundation**

```bash
git add src/main.ts src/lib/theme.css src/lib/icons.ts src/lib/components/Button.svelte src/lib/components/Badge.svelte src/lib/components/Panel.svelte src/lib/components/StatusRow.svelte src/lib/components/EmptyState.svelte src/lib/components/ErrorPanel.svelte src/lib/components/__tests__/primitives.test.ts src/assets/fonts/Geist-Regular.woff2 src/assets/fonts/Geist-SemiBold.woff2 src/assets/fonts/JetBrainsMono-Regular.woff2 src/assets/fonts/JetBrainsMono-Medium.woff2
git diff --cached --check
git commit -m "feat: add Stitch visual foundation"
```

---

### Task 3: Add session state, view models, and shell navigation

**Files:**
- Create: `src/lib/view-models.ts`
- Create: `src/lib/components/AppShell.svelte`
- Create: `src/lib/components/Sidebar.svelte`
- Create: `src/lib/components/TopBar.svelte`
- Create: `src/lib/components/__tests__/view-models.test.ts`
- Create: `src/lib/components/__tests__/AppShell.test.ts`
- Modify: `src/lib/types.ts`
- Modify: `src/lib/stores.ts`
- Modify: `src/lib/notifications.ts`
- Modify: `src/App.svelte`

**Interfaces:**
- Extend `AppSettings` with `default_layout_id: string | null` to match the existing Rust field.
- `AppView = "workspace" | "layouts" | "settings" | "status"`.
- `InitializationState = { status: "loading" } | { status: "loaded"; state: FrontendState } | { status: "failed"; message: string }`.
- `view-models.ts` exports `isUsableLayout(layout, monitors): boolean`, `getFirstRunState(frontendState): "onboarding" | "empty" | "recovery" | "ready"`, `isDefaultLayout(layout, settings): boolean`, and `isActiveLayout(layout, activeLayouts): boolean`.
- Notification API exposes separate `toastNotifications` and `notificationHistory` stores. Toasts expire after five seconds; history retains at most 100 entries, newest first, coalesces consecutive identical `{message,type}`, and exposes `clearNotificationHistory()`.
- `AppShell.svelte` props: `{ activeView: AppView; initialization: InitializationState; isPaused: boolean; monitorCount: number; onNavigate: (view: AppView) => void; onRetry: () => void }`.

- [ ] **Step 1: Write view-model and notification tests**

Cover the three first-run states, layout usability requiring at least one zone and matching monitor, default/active badge derivation, 100-entry history cap, duplicate coalescing, and clear behavior.

Use a shared typed fixture and exact assertions:

```ts
const state: FrontendState = {
  monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
  active_layouts: [], saved_layouts: [], is_paused: false,
  settings: { auto_start: false, default_gap: 4, default_margin: 8, accent_color: "#8B5CF6", language: "en", first_run_completed: false, default_layout_id: null },
};
expect(getFirstRunState(state)).toBe("onboarding");
expect(isUsableLayout({ zones: [], monitor_id: "m1" }, state.monitors)).toBe(false);

for (let index = 0; index < 101; index += 1) notify(`message-${index}`, "info");
expect(get(notificationHistory)).toHaveLength(100);
clearNotificationHistory();
expect(get(notificationHistory)).toEqual([]);
```

- [ ] **Step 2: Implement view models and extend types**

Use `settings.default_layout_id` for default state and `active_layouts` monitor IDs for active state. Do not add Rust commands or change the config schema.

- [ ] **Step 3: Implement bounded notifications**

Implement separate toast and history lifecycles. `notify()` appends to both stores; the five-second timer removes only the toast entry. Route `user-notification` events through `notify()`, and make System Status consume only `notificationHistory`.

- [ ] **Step 4: Implement `Sidebar`, `TopBar`, and `AppShell`**

Use a 280px fixed sidebar, primary navigation for Workspace/Saved Layouts/Settings, a clickable runtime status affordance opening `status`, and `Back to Workspace` in the dedicated status view. Keep all interactive elements keyboard reachable.

- [ ] **Step 5: Update `App.svelte` startup state machine**

Render shell immediately in `loading`; show skeleton route content; on `getCurrentState` failure render shell plus Retry and status error; on success choose onboarding/empty/recovery/ready using view models. Keep navigation available in all states.

- [ ] **Step 6: Run shell tests and build**

```bash
npx vitest run src/lib/components/__tests__/view-models.test.ts src/lib/components/__tests__/AppShell.test.ts
npm run build
```

Expected: tests pass and Vite build completes.

- [ ] **Step 7: Commit shell/state foundation**

```bash
git add src/App.svelte src/lib/types.ts src/lib/stores.ts src/lib/notifications.ts src/lib/view-models.ts src/lib/components/AppShell.svelte src/lib/components/Sidebar.svelte src/lib/components/TopBar.svelte src/lib/components/__tests__/view-models.test.ts src/lib/components/__tests__/AppShell.test.ts
git diff --cached --check
git commit -m "feat: add Stitch app shell and state model"
```

---

### Task 4: Rebuild Workspace and First-run/Empty State

**Files:**
- Create: `src/lib/components/MonitorCanvas.svelte`
- Create: `src/lib/components/ZoneInspector.svelte`
- Create: `src/routes/__tests__/WorkspaceStates.test.ts`
- Modify: `src/routes/LayoutEditor.svelte`
- Modify: `src/routes/__tests__/LayoutEditor.test.ts`
- Modify: `src/App.svelte`

**Interfaces:**
- `MonitorCanvas.svelte` props: `{ monitor: Monitor; draftZones: Zone[]; selectedZoneId: string | null; onCreateZone: (monitorId: string, rect: { x: number; y: number; width: number; height: number }) => void; onSelectZone: (zoneId: string | null) => void; onChangeZone: (zoneId: string, patch: Partial<Zone>) => void; onDeleteZone: (zoneId: string) => void }`.
- `ZoneInspector.svelte` props: `{ zone: Zone | null; onRename: (zoneId: string, name: string) => void; onDelete: (zoneId: string) => void; onChange: (zoneId: string, patch: Partial<Zone>) => void }`.
- Editor route keeps `applyLayout(layout: Layout)` and `saveLayout(name, zones, monitorId)` as layout persistence calls, uses `getCurrentState(): Promise<FrontendState>` after success, and uses `saveSettings(settings: AppSettings)` only for the explicit `Finish setup` action.

- [ ] **Step 1: Update tests for local draft behavior**

Keep current monitor rendering, create-zone, delete-confirmation, and keyboard tests. Mock `applyLayout`/`saveLayout` with `vi.fn()` and rejected promises; assert draft changes do not call IPC, Apply Live calls once per monitor, pending buttons disable, controls re-enable after resolve/reject, drafts remain unchanged after rejection, and an error notification is added. Add tests for no monitors and all three first-run states.

Use a deferred promise to make pending behavior deterministic:

```ts
it("applies only on explicit action and preserves draft after failure", async () => {
  const apply = vi.mocked(applyLayout);
  apply.mockRejectedValueOnce(new Error("apply failed"));
  const view = render(LayoutEditor);
  const canvas = view.container.querySelector(".monitor-canvas")!;
  await fireEvent.pointerDown(canvas, { clientX: 120, clientY: 80 });
  expect(apply).not.toHaveBeenCalled();
  await fireEvent.click(view.getByRole("button", { name: "Apply Live" }));
  await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
  expect(view.getByRole("region", { name: /Zone 1/ })).toBeTruthy();
  expect(await view.findByText(/apply failed/i)).toBeTruthy();
  expect((view.getByRole("button", { name: "Apply Live" }) as HTMLButtonElement).disabled).toBe(false);
});
```

- [ ] **Step 2: Extract monitor canvas and inspector**

Move canvas rendering into `MonitorCanvas.svelte`, isolate pointer/keyboard state to the canvas and selected inspector, use CSS/SVG for static grid/rulers, and preserve 12-column fractional coordinate calculations. Pointer handlers update draft state immediately; dynamic preview/ruler repaint is scheduled once per `requestAnimationFrame`, while unchanged labels, sidebar, toolbar, and settings panels stay outside that update path.

- [ ] **Step 3: Implement Stitch Workspace composition**

Build the technical toolbar, monitor preview panels, zone labels, coordinate/dimension metadata, selected-zone inspector, Save and Apply Live controls, and minimum-size scrolling behavior. Use local draft state until explicit persistence. After successful Apply Live or Save, call `getCurrentState()` and replace `currentState`, `savedLayouts`, and related route data; after failure preserve the draft and expose Retry/error feedback.

- [ ] **Step 4: Implement explicit first-run/empty/recovery states**

Use `EmptyState.svelte` for onboarding, empty workspace, and layout recovery with distinct copy. In onboarding, `Create your first layout` navigates to Workspace and does not modify `first_run_completed`; a separate `Finish setup` action calls `saveSettings({ ...settings, first_run_completed: true })`. Add a test that clicks each action and asserts navigation versus exactly one settings persistence call.

- [ ] **Step 5: Run Workspace tests**

```bash
npx vitest run src/routes/__tests__/LayoutEditor.test.ts src/routes/__tests__/WorkspaceStates.test.ts
```

Expected: all editor and state tests pass.

- [ ] **Step 6: Commit Workspace UI**

```bash
git add src/App.svelte src/routes/LayoutEditor.svelte src/routes/__tests__/LayoutEditor.test.ts src/routes/__tests__/WorkspaceStates.test.ts src/lib/components/MonitorCanvas.svelte src/lib/components/ZoneInspector.svelte
git diff --cached --check
git commit -m "feat: rebuild Stitch workspace editor"
```

---

### Task 5: Rebuild Saved Layouts with real badges and thumbnails

**Files:**
- Create: `src/lib/components/LayoutThumbnail.svelte`
- Create: `src/routes/__tests__/LayoutManager.test.ts`
- Modify: `src/routes/LayoutManager.svelte`
- Modify: `src/lib/view-models.ts`

**Interfaces:**
- `LayoutThumbnail.svelte` props: `{ layoutId: string; zones: Zone[]; label: string }`; renders normalized zone rectangles as escaped/local DOM geometry, not remote images. Memoization keys on `layoutId` plus a stable serialized zone signature.
- Route mutation flow: `setDefaultLayout/deleteLayout` → await IPC → `getCurrentState()` → update `currentState` and `savedLayouts` stores.

- [ ] **Step 1: Write Saved Layouts tests**

Mock `listLayouts`, `setDefaultLayout`, `deleteLayout`, and `getCurrentState`. Test loading, empty state, layout name/zone count, default badge from `settings.default_layout_id`, active badge from `active_layouts`, successful mutation refresh, confirmation, pending state, duplicate-click suppression, control re-enable after resolve/reject, and error notification. Use before/after `getCurrentState` fixtures and assert `savedLayouts`/`currentState` store replacement.

Representative mutation assertion:

```ts
it("refreshes default badge after setting a default", async () => {
  vi.mocked(listLayouts).mockResolvedValue([layoutFixture]);
  vi.mocked(setDefaultLayout).mockResolvedValue();
  vi.mocked(getCurrentState).mockResolvedValue({ ...stateFixture, settings: { ...stateFixture.settings, default_layout_id: layoutFixture.id } });
  const view = render(LayoutManager);
  await fireEvent.click(await view.findByRole("button", { name: "Set Default" }));
  await waitFor(() => expect(setDefaultLayout).toHaveBeenCalledWith(layoutFixture.id));
  expect(await view.findByText("DEFAULT")).toBeTruthy();
});
```

- [ ] **Step 2: Implement normalized thumbnails**

Render each zone from fractional `x/y/width/height`, memoizing by layout ID plus a stable zone signature. Use a panel background and dashed zone borders matching the Stitch treatment.

- [ ] **Step 3: Implement the Stitch layout list/cards**

Use Panel/Badge/Button primitives, technical metadata, active/default badges, Set Default and Delete actions, empty CTA back to Workspace, and escaped layout names.

- [ ] **Step 4: Refresh stores after mutations and run tests**

```bash
npx vitest run src/routes/__tests__/LayoutManager.test.ts
```

Expected: all Saved Layouts tests pass.

- [ ] **Step 5: Commit Saved Layouts UI**

```bash
git add src/routes/LayoutManager.svelte src/routes/__tests__/LayoutManager.test.ts src/lib/components/LayoutThumbnail.svelte src/lib/view-models.ts
git commit -m "feat: rebuild Stitch saved layouts view"
```

---

### Task 6: Rebuild Settings and add dedicated System Status

**Files:**
- Create: `src/routes/SystemStatus.svelte`
- Create: `src/routes/__tests__/Settings.test.ts`
- Create: `src/routes/__tests__/SystemStatus.test.ts`
- Modify: `src/routes/Settings.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/notifications.ts`

**Interfaces:**
- `SystemStatus.svelte` consumes `{ state: FrontendState | null; initialization: InitializationState; history: Notification[]; onRetry: () => void; onClearHistory: () => void }`.
- Settings save flow: `saveSettings(settings)` → `getCurrentState()` → update `currentState`, `settings`, and `savedLayouts` stores; failed save leaves edited values and shows ErrorPanel/toast.

- [ ] **Step 1: Write Settings and Status tests**

Test settings loading/loaded/error states, all existing fields including `default_layout_id` preservation, save success/failure, disabled Save during pending, one-call-only duplicate-click suppression, re-enable after resolve/reject, status monitor/paused rows, notification ordering, Clear history, initialization error and Retry, and dedicated Back to Workspace action.

Representative failure assertion:

```ts
it("retains edited settings and restores Save after failure", async () => {
  vi.mocked(getSettings).mockResolvedValue(settingsFixture);
  vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));
  const view = render(Settings);
  const gap = await view.findByLabelText("Default gap between zones (px)");
  await fireEvent.input(gap, { target: { value: "12" } });
  await fireEvent.click(view.getByRole("button", { name: "Save Settings" }));
  expect(await view.findByText(/save failed/i)).toBeTruthy();
  expect((gap as HTMLInputElement).value).toBe("12");
  expect((view.getByRole("button", { name: "Save Settings" }) as HTMLButtonElement).disabled).toBe(false);
});
```

- [ ] **Step 2: Recompose Settings with Stitch panels**

Group runtime, layout defaults, appearance, language, and about information using shared controls. Preserve `auto_start`, gap, margin, accent color, language, first-run, and default layout values. Use English labels and visible save feedback.

- [ ] **Step 3: Implement `SystemStatus.svelte`**

Render a dedicated non-modal view with title, Back to Workspace, runtime status, monitor count, recent notifications, safe fallback labels, Retry for initialization failure, and Clear history. Do not add backend health commands.

- [ ] **Step 4: Run tests**

```bash
npx vitest run src/routes/__tests__/Settings.test.ts src/routes/__tests__/SystemStatus.test.ts
```

Expected: all Settings and Status tests pass.

- [ ] **Step 5: Commit Settings and Status**

```bash
git add src/routes/Settings.svelte src/routes/SystemStatus.svelte src/routes/__tests__/Settings.test.ts src/routes/__tests__/SystemStatus.test.ts src/App.svelte src/lib/notifications.ts
git diff --cached --check
git commit -m "feat: add Stitch settings and system status"
```

---

### Task 7: Security, accessibility, responsive, and packaged-app validation

**Files:**
- Modify: `src-tauri/tauri.conf.json` only if the existing restrictive CSP/capabilities need tightening; do not add remote permissions
- Modify: `src-tauri/src/lib.rs` only to add missing argument validation helpers without changing command signatures
- Modify: `src-tauri/src/config_store.rs` only if persisted settings validation is incomplete
- Modify: `src-tauri/tests/security_smoke.rs`
- Modify: `src-tauri/tests/config_store_tests.rs`
- Create: `src/lib/__tests__/security-rendering.test.ts`
- Create: `docs/superpowers/references/stitch-grid-screen/visual-checklist.md`
- Modify: `README.md` with required local development, packaging, smoke-test, performance-baseline, and rollback instructions

**Interfaces:**
- No new IPC commands.
- No new remote permissions or runtime network dependencies.
- Visual checklist covers all five user-facing screens at 2560×2048 and the 1024×720 minimum.

- [ ] **Step 1: Add malicious-text rendering tests**

Render layout names, monitor names, technical metadata, settings-derived labels, and notifications containing `<img src=x onerror=alert(1)>`. Assert the exact string is present as text, `container.querySelector("img")` is null, and `rg -n '\{@html' src` returns no dynamic raw-HTML use. Pass `NaN`, `Infinity`, negative, and values above 1.0 into thumbnail fixtures and assert generated style/SVG geometry is clamped or omitted rather than serialized as unsafe/non-finite attributes.

- [ ] **Step 2: Run accessibility checks**

Use Testing Library keyboard interactions to verify sidebar order, visible focus, dialog focus trap/restore, icon-only accessible names, and live-region status messages. Inspect `theme.css` for `@media (prefers-reduced-motion: reduce)` and assert transition/animation duration is disabled or reduced under that media rule.

- [ ] **Step 3: Validate the existing CSP and capability boundary**

Confirm `tauri.conf.json` keeps `default-src 'self'`, no remote script/frame/image/font source, and existing capability permissions only. If changing CSP, run the packaged app and verify no local asset/font is blocked.

- [ ] **Step 4: Add visual checklist and manual comparison**

Create a screen-specific checklist for Workspace, Saved Layouts, Settings, First-run/Empty, and System Status. Capture implementation screenshots at 2560×2048 and 1024×720, then record pass/fail notes for sidebar width, typography hierarchy, panel alignment, spacing, canvas proportions, zone/ruler treatment, primary actions, and permitted deviations.

- [ ] **Step 5: Run full frontend validation**

```bash
npx vitest run
npm run build
```

Expected: all Vitest tests pass and Vite build completes without remote asset warnings.

- [ ] **Step 6: Write failing Rust trust-boundary tests**

Add negative tests for non-finite/out-of-range zone coordinates and dimensions, empty/oversized layout names, excessive gap/margin, malformed accent colors, unsupported language values, invalid default layout IDs, and unsafe auto-start/settings inputs. Assert each malformed value is rejected before persistence or privileged platform action, while existing valid fixtures still pass.

- [ ] **Step 7: Implement missing Rust validation without changing IPC contracts**

Inspect existing validators first. Add only missing checks in the command/config boundary: finite geometry in `0.0..=1.0`, positive zone extents that remain within the monitor fraction, bounded names (1–64 characters), gap/margin `0..=100`, accent colors matching `#[0-9A-Fa-f]{6}`, language in `{en, vi}`, and a default layout ID that is null or references an existing layout. Preserve command names, argument shapes, config schema version, and valid behavior.

- [ ] **Step 8: Run backend regression tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: new malformed-input tests and all existing Rust tests pass without command/signature/schema changes.

- [ ] **Step 9: Record the editor performance baseline**

Use a representative workload of four monitors with 24 total zones at 2560×2048 and 1024×720. Record OS/windowing system, CPU, window size, monitor count, and zone count in `visual-checklist.md`. Verify with IPC mocks or Tauri logs that pointer movement performs zero `applyLayout`/`saveLayout` calls, inspect browser performance traces for one preview repaint per animation frame and no visible drag stutter, then observe idle CPU for 60 seconds after editing/navigation and confirm the refresh introduces no sustained elevation above the current app baseline.

- [ ] **Step 10: Build, inspect, and smoke-test packaged Tauri app**

```bash
npm run tauri build
```

Validate Linux X11 and Windows as supported release targets; record any target that cannot be executed in the current environment as externally pending rather than silently passing it. Search `dist/`, generated Tauri resources, and package contents for Stitch-hosted URLs, downloaded HTML/scripts, and unapproved assets. Launch the package in a clean environment and verify local fonts/assets, startup states, first-run completion, Retry, Workspace editing, one IPC-backed action, and representative existing config compatibility.

Record the previous artifact version and absolute retention location. Launch that previous build against a copy of the representative config, document any downgrade limitation, then relaunch the new build to prove the rollback path is usable and non-destructive.

- [ ] **Step 11: Commit validation and documentation**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/lib.rs src-tauri/src/config_store.rs src-tauri/tests/security_smoke.rs src-tauri/tests/config_store_tests.rs src/lib/__tests__/security-rendering.test.ts docs/superpowers/references/stitch-grid-screen/visual-checklist.md README.md
git diff --cached --check
git commit -m "test: validate Stitch UI refresh"
```

---

## Plan Self-Review

- Spec coverage: all six Stitch references, shell/navigation, tokens, first-run state matrix, Workspace draft/apply behavior, Saved Layouts badges/thumbnails, Settings, dedicated Status, startup loading/retry, bounded notifications, accessibility, safe rendering, Rust trust boundary, local fonts, packaged smoke test, rollback, and 1024×720 validation map to Tasks 1–7.
- Placeholder scan: no unfinished requirement or unspecified implementation branch remains. The hosted URL values are source metadata supplied during implementation, not runtime dependencies; the reference manifest task identifies exactly where they must be recorded.
- Type consistency: `AppSettings.default_layout_id`, `AppView`, `InitializationState`, view-model functions, component props, and mutation refresh flow are defined once in Task 3 and reused by Tasks 4–6.
- Scope: no Rust feature, new IPC command, database, auth, telemetry, or unrelated refactor is included.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-10-stitch-ui-refresh-implementation.md`. Two execution options:

1. **Subagent-Driven (recommended)** — dispatch a fresh worker per task and review between tasks.
2. **Inline Execution** — execute the tasks in this session with checkpoints.

Choose one approach before implementation begins.
