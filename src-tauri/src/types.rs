use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: MonitorId,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub dpi_scale: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub handle: WindowHandle,
    pub title: String,
    pub rect: Rect,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub u64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: Uuid,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub gap: u32,
    pub margin: u32,
}

impl Zone {
    pub fn effective_rect(&self, monitor: &Monitor) -> Rect {
        let mx = monitor.x as f64 + self.x * monitor.width as f64 + self.margin as f64 + (self.gap as f64 / 2.0);
        let my = monitor.y as f64 + self.y * monitor.height as f64 + self.margin as f64 + (self.gap as f64 / 2.0);
        let mw = (self.width * monitor.width as f64) - 2.0 * self.margin as f64 - self.gap as f64;
        let mh = (self.height * monitor.height as f64) - 2.0 * self.margin as f64 - self.gap as f64;
        Rect {
            x: mx.floor() as i32,
            y: my.floor() as i32,
            width: (mw.floor() as u32).max(1),
            height: (mh.floor() as u32).max(1),
        }
    }

    pub fn contains(&self, px: f64, py: f64, monitor: &Monitor) -> bool {
        let ex = self.x * monitor.width as f64;
        let ey = self.y * monitor.height as f64;
        let ew = self.width * monitor.width as f64;
        let eh = self.height * monitor.height as f64;
        px >= ex && px <= ex + ew && py >= ey && py <= ey + eh
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub zones: Vec<Zone>,
    pub monitor_id: MonitorId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    pub id: Uuid,
    pub name: String,
    pub arrangement_id: String,
    pub zones: Vec<Zone>,
    pub monitor_id: MonitorId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub schema_version: u32,
    pub layouts: Vec<SavedLayout>,
    pub settings: AppSettings,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            layouts: vec![],
            settings: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_start: bool,
    pub default_gap: u32,
    pub default_margin: u32,
    pub accent_color: String,
    pub language: String,
    pub first_run_completed: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            default_gap: 4,
            default_margin: 8,
            accent_color: "#7C3AED".into(),
            language: "en".into(),
            first_run_completed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub window_handle: WindowHandle,
    pub original_rect: Rect,
    pub snap_in_progress: bool,
}

#[derive(Debug, Clone)]
pub enum WindowMoveEvent {
    DragStart { handle: WindowHandle, rect: Rect },
    DragMove { handle: WindowHandle, rect: Rect },
    DragEnd { handle: WindowHandle, rect: Rect },
}

#[derive(Debug, Clone)]
pub enum DisplayChangeEvent {
    Connected,
    Disconnected,
    ResolutionChanged,
}

#[derive(Debug, Clone)]
pub struct SnapEvent {
    pub window_handle: WindowHandle,
    pub zone_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverlayHandle(pub u64);
