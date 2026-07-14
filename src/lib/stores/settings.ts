import { writable } from "svelte/store";
import type { Settings } from "../shared-types";

const defaults: Settings = {
  schema_version: 1,
  snap_enabled: true,
  snap_modifier: "Shift",
  autostart_enabled: false,
  minimize_to_tray: true,
  last_layout_id: null,
  active_target_screen_hint: null,
  default_gap_px: 10,
  default_margin_px: 16,
};

export const settings = writable<Settings>(defaults);
