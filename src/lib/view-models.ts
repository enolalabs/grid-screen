import type { FrontendState, Monitor, Layout, AppSettings } from "./types";

export function isUsableLayout(layout: { zones: unknown[]; monitor_id: string }, monitors: Monitor[]): boolean {
  if (layout.zones.length === 0) return false;
  return monitors.some(m => m.id === layout.monitor_id);
}

export function getFirstRunState(state: FrontendState): "onboarding" | "empty" | "recovery" | "ready" {
  if (!state.settings.first_run_completed) return "onboarding";
  if (state.saved_layouts.length === 0) return "empty";
  if (state.monitors.length === 0) return "recovery";
  return "ready";
}

export function isDefaultLayout(layout: { id: string }, settings: AppSettings): boolean {
  return settings.default_layout_id === layout.id;
}

export function isActiveLayout(layout: { monitor_id: string }, activeLayouts: Layout[]): boolean {
  return activeLayouts.some(al => al.monitor_id === layout.monitor_id);
}
