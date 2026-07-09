import { writable } from "svelte/store";

export interface Notification {
  id: string;
  message: string;
  type: "info" | "warning" | "error";
}
export const notifications = writable<Notification[]>([]);

export function notify(message: string, type: "info" | "warning" | "error" = "info") {
  const id = crypto.randomUUID();
  notifications.update(n => [...n, { id, message, type }]);
  setTimeout(() => {
    notifications.update(n => n.filter(x => x.id !== id));
  }, 5000);
}
