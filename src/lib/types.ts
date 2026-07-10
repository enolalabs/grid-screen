export interface Monitor {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  dpi_scale: number;
  is_primary: boolean;
}

export interface Zone {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  gap: number;
  margin: number;
}

export interface Layout {
  zones: Zone[];
  monitor_id: string;
}

export interface SavedLayout {
  id: string;
  name: string;
  arrangement_id: string;
  zones: Zone[];
  monitor_id: string;
}

export interface AppSettings {
  auto_start: boolean;
  default_gap: number;
  default_margin: number;
  accent_color: string;
  language: string;
  first_run_completed: boolean;
  default_layout_id: string | null;
}

export interface FrontendState {
  monitors: Monitor[];
  active_layouts: Layout[];
  saved_layouts: SavedLayout[];
  is_paused: boolean;
  settings: AppSettings;
}

export type AppView = "workspace" | "layouts" | "settings" | "status";

export type InitializationState =
  | { status: "loading" }
  | { status: "loaded"; state: FrontendState }
  | { status: "failed"; message: string };
