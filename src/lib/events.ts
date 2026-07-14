import { listen } from "@tauri-apps/api/event";
import type { WorkspaceChangedPayload, ScreenChangedPayload, SystemStatus } from "./shared-types";
import { clearAssignments } from "./stores/assignments";
import { showToast } from "./stores/toasts";
import { windows } from "./stores/windows";
import { commands } from "./commands";
import { systemStatus } from "./stores/systemStatus";

export function registerEventListeners() {
  const unlistenWorkspace = listen<WorkspaceChangedPayload>(
    "workspace-changed",
    () => {
      clearAssignments();
      commands.refreshWindows().then((w) => windows.set(w));
      showToast("Workspace changed — assignments cleared", "warning");
    }
  );

  const unlistenScreen = listen<ScreenChangedPayload>(
    "screen-changed",
    (_event) => {
      showToast("Screen configuration changed", "warning");
    }
  );

  const unlistenSystemStatus = listen<SystemStatus>(
    "system-status-changed",
    (event) => {
      systemStatus.set(event.payload);
    }
  );

  return () => {
    unlistenWorkspace.then((fn) => fn());
    unlistenScreen.then((fn) => fn());
    unlistenSystemStatus.then((fn) => fn());
  };
}