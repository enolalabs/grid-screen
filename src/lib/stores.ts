import { writable } from "svelte/store";
import type { FrontendState, Monitor, Layout, SavedLayout, AppSettings } from "./types";

export const currentState = writable<FrontendState | null>(null);
export const selectedMonitor = writable<Monitor | null>(null);
export const activeLayout = writable<Layout | null>(null);
export const savedLayouts = writable<SavedLayout[]>([]);
export const settings = writable<AppSettings | null>(null);
