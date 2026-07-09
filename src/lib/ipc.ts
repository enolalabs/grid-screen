import { invoke } from "@tauri-apps/api/core";
import type { FrontendState, Layout, SavedLayout, Zone, AppSettings } from "./types";

export async function getCurrentState(): Promise<FrontendState> {
  return await invoke<FrontendState>("get_current_state");
}

export async function applyLayout(layout: Layout): Promise<void> {
  await invoke("apply_layout", { layout });
}

export async function saveLayout(name: string, zones: Zone[], monitorId: string): Promise<void> {
  await invoke("save_layout", { name, zones, monitorId });
}

export async function listLayouts(): Promise<SavedLayout[]> {
  return await invoke<SavedLayout[]>("list_layouts");
}

export async function deleteLayout(id: string): Promise<void> {
  await invoke("delete_layout", { id });
}

export async function togglePause(): Promise<boolean> {
  return await invoke<boolean>("toggle_pause");
}

export async function getSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke("save_settings", { settings });
}
