# Grid Screen — Stitch UI Refresh Visual Checklist

| Screen | Status | Notes |
|--------|--------|-------|
| Workspace | Pending | Layout editor: zone canvas, inspector, toolbar at 2560x2048 and 1024x720 |
| Saved Layouts | Pending | Layout list with thumbnails, badges, and delete/edit actions |
| Settings | Pending | Settings form: auto-start, gap, margin, accent color picker, language selector |
| First-run / Empty | Pending | Onboarding dialog, empty-state illustrations, CTA buttons |
| System Status | Pending | Runtime info panel, notification history list, clear action |

### Layout and spacing checklist

| Property | Expected | Status |
|----------|----------|--------|
| Sidebar width | 280px | Pending |
| Sidebar is not collapsible below 280px | Fixed at min 280px | Pending |
| Canvas background | #0F0D15 | Pending |
| Surface backgrounds | #1D1A23 | Pending |
| Primary accent | #8B5CF6 | Pending |
| Base spacing unit | 4px | Pending |
| Control height | 32px | Pending |
| Panel border-radius | 8px | Pending |
| Control border-radius | 4px | Pending |
| Focus ring | `0 0 0 2px #0F0D15, 0 0 0 4px #D0BCFF` | Pending |
| Font family (body) | Geist | Pending |
| Font family (mono) | JetBrains Mono | Pending |
| Dark color scheme | `color-scheme: dark` | Pending |

### Responsive behavior

| Breakpoint | Expected | Status |
|------------|----------|--------|
| 2560x2048 | Full workspace with inspector sidebar visible | Pending |
| 1024x720 | Compact layout, scrollable zones, inspector may stack | Pending |

### Accessibility

| Feature | Expected | Status |
|---------|----------|--------|
| Reduced motion | Transitions/animations disabled under `prefers-reduced-motion: reduce` | Pending |
| Focus-visible | Purple ring via `var(--focus-ring)` on focused elements | Pending |
| Sidebar role | `role="navigation"` with `aria-label="Main navigation"` | Pending |
| Nav tabs | `role="tab"` with `aria-selected` state | Pending |
| Live regions | Toast container has `role="status" aria-live="polite"` | Pending |
| Alert role | Error panels use `role="alert"` | Pending |
| Accessible names | All nav buttons and controls have text or aria-label | Pending |

### Performance baseline (target)

- Four monitors with 24 total zones at 2560x2048 and 1024x720
- Zero `applyLayout`/`saveLayout` calls during pointer movement
- One preview repaint per animation frame
- No visible drag stutter
- Idle CPU for 60s after editing: no sustained elevation above baseline

### Notes

- Target expectations: "Four monitors with 24 total zones at 2560x2048 and 1024x720 — zero applyLayout/saveLayout calls during pointer movement, one preview repaint per animation frame."
- Performance baseline not recorded in this environment (no Tauri build available for cross-compilation). Expected behavior verified vicariously through IPC mock tests confirming applyLayout/saveLayout are not called during pointer/drag events.

### Security

| Check | Expected | Status |
|-------|----------|--------|
| CSP | `default-src 'self'`, no `unsafe-eval` | Pass |
| Capabilities | No shell, HTTP, or filesystem permissions | Pass |
| XSS prevention | No `{@html}` in runtime sources | Verified |
| Config persistence | `0o600` permissions on saved layouts | Pass |
| Input validation | Zone coords finite 0-1, names 1-64 chars, gap/margin 0-100, accent `#[0-9A-Fa-f]{6}`, language `{en,vi}` | Pending |
