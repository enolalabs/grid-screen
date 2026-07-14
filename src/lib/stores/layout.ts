import { writable, derived } from "svelte/store";
import type { Layout } from "../shared-types";

export const layouts = writable<Layout[]>([]);
export const selectedLayoutId = writable<string>("");
export const sessionOverrides = writable<{
  ratio?: number;
  gap_px?: number;
  margin_px?: number;
}>({});

export function selectLayout(id: string) {
  selectedLayoutId.set(id);
  sessionOverrides.set({});
}

export const selectedLayout = derived(
  [layouts, selectedLayoutId, sessionOverrides],
  ([$layouts, $selectedLayoutId, $sessionOverrides]) => {
    const base = $layouts.find((l) => l.id === $selectedLayoutId);
    if (!base) return null;
    return { ...base, ...$sessionOverrides };
  }
);
