import { writable } from "svelte/store";
import type { SystemStatus } from "../shared-types";

export const systemStatus = writable<SystemStatus>({
  session_type: "unknown",
  ewmh_support: "unknown",
  wm_name: "unknown",
  xrandr_available: false,
  workspace: "unknown",
  connected_screens: "unknown",
  errors: [],
});
