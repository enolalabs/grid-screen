import { writable, get } from "svelte/store";

export interface Notification {
  id: string;
  message: string;
  type: "info" | "warning" | "error";
}

export const toastNotifications = writable<Notification[]>([]);
export const notificationHistory = writable<Notification[]>([]);

export function notify(message: string, type: "info" | "warning" | "error" = "info") {
  const id = crypto.randomUUID();
  const entry: Notification = { id, message, type };

  toastNotifications.update(n => [...n, entry]);
  setTimeout(() => {
    toastNotifications.update(n => n.filter(x => x.id !== id));
  }, 5000);

  notificationHistory.update(n => {
    const latest = n[0];
    if (latest && latest.message === message && latest.type === type) return n;
    const next = [{ ...entry, id: crypto.randomUUID() }, ...n];
    if (next.length > 100) next.length = 100;
    return next;
  });
}

export function clearNotificationHistory() {
  notificationHistory.set([]);
}
