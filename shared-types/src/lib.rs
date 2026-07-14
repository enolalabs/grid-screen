use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenInfo {
    pub id: String,
    pub label: String,
    pub resolution: String,
    pub work_area: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WindowDescriptor {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub icon_color: String,
    pub state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WindowState {
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub movable: bool,
    pub resizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Layout {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub layout_type: LayoutType,
    pub zones: u32,
    pub columns: String,
    pub rows: Option<String>,
    pub span_first: Option<bool>,
    pub ratio: Option<u32>,
    pub gap_px: u32,
    pub margin_px: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum LayoutType {
    Preset,
    Saved,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Settings {
    pub schema_version: u32,
    pub snap_enabled: bool,
    pub snap_modifier: String,
    pub autostart_enabled: bool,
    pub minimize_to_tray: bool,
    pub last_layout_id: Option<String>,
    pub active_target_screen_hint: Option<String>,
    pub default_gap_px: u32,
    pub default_margin_px: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SystemStatus {
    pub session_type: String,
    pub ewmh_support: String,
    pub wm_name: String,
    pub xrandr_available: bool,
    pub workspace: String,
    pub connected_screens: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArrangeRequest {
    pub layout_id: String,
    pub screen_id: String,
    pub assignments: HashMap<u32, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArrangeResult {
    pub success: bool,
    pub results: Vec<PerWindowResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerWindowResult {
    pub window_id: String,
    pub status: MoveStatus,
    pub actual_rect: Option<Rect>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum MoveStatus {
    Moved,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BootstrapData {
    pub screens: Vec<ScreenInfo>,
    pub layouts: Vec<Layout>,
    pub windows: Vec<WindowDescriptor>,
    pub settings: Settings,
    pub system_status: SystemStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkspaceChangedPayload {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenChangedPayload {
    pub screens: Vec<ScreenInfo>,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            minimized: false,
            maximized: false,
            fullscreen: false,
            movable: true,
            resizable: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: 1,
            snap_enabled: true,
            snap_modifier: "Shift".to_string(),
            autostart_enabled: false,
            minimize_to_tray: true,
            last_layout_id: None,
            active_target_screen_hint: None,
            default_gap_px: 10,
            default_margin_px: 16,
        }
    }
}

impl Default for SystemStatus {
    fn default() -> Self {
        SystemStatus {
            session_type: "unknown".to_string(),
            ewmh_support: "unknown".to_string(),
            wm_name: "unknown".to_string(),
            xrandr_available: false,
            workspace: "unknown".to_string(),
            connected_screens: "unknown".to_string(),
            errors: Vec::new(),
        }
    }
}
