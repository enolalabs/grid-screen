import { writable } from "svelte/store";

export interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "warning";
  durationMs: number;
}

export const toasts = writable<Toast[]>([]);

export function showToast(
  message: string,
  type: Toast["type"] = "success",
  durationMs = 3000
) {
  const id = crypto.randomUUID();
  toasts.update((t) => [...t, { id, message, type, durationMs }]);
  if (durationMs > 0) {
    setTimeout(() => dismissToast(id), durationMs);
  }
}

export function dismissToast(id: string) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
