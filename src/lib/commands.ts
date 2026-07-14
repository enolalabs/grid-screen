import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapData,
  WindowDescriptor,
  Layout,
  ArrangeRequest,
  ArrangeResult,
  Settings,
} from "./shared-types";

export const commands = {
  bootstrap: () => invoke<BootstrapData>("bootstrap"),
  refreshWindows: () => invoke<WindowDescriptor[]>("refresh_windows"),
  arrangeWindows: (req: ArrangeRequest) =>
    invoke<ArrangeResult>("arrange_windows", { request: req }),
  saveLayout: (layout: Layout) => invoke<void>("save_layout", { layout }),
  deleteLayout: (layoutId: string) =>
    invoke<void>("delete_layout", { layoutId }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<void>("update_settings", { settings }),
  saveDefaults: (gapPx: number, marginPx: number) =>
    invoke<void>("save_defaults", { gapPx, marginPx }),
  getDiagnostics: () => invoke<string>("get_diagnostics"),
};