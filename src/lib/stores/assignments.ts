import { writable, derived } from "svelte/store";

export const assignments = writable<Record<number, string>>({});

export const assignedWindowIds = derived(assignments, ($a) =>
  new Set(Object.values($a))
);

export const assignedCount = derived(assignments, ($a) =>
  Object.keys($a).length
);

export function assignWindow(zoneIndex: number, windowId: string) {
  assignments.update((a) => {
    const cleaned: Record<number, string> = {};
    for (const [z, wid] of Object.entries(a)) {
      if (wid !== windowId) cleaned[Number(z)] = wid;
    }
    cleaned[zoneIndex] = windowId;
    return cleaned;
  });
}

export function removeWindowFromZone(zoneIndex: number) {
  assignments.update((a) => {
    const next = { ...a };
    delete next[zoneIndex];
    return next;
  });
}

export function clearAssignments() {
  assignments.set({});
}
