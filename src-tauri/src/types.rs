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

impl MonitorId {
    /// Create a stable, deterministic ID from the monitor's unique name.
    /// Uses UUID v5 (namespace-based) so the same monitor always gets the same ID.
    pub fn from_name(name: &str) -> Self {
        // Custom namespace for Grid Screen monitor IDs
        const GS_MONITOR_NS: Uuid = Uuid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ]);
        Self(uuid::Uuid::new_v5(&GS_MONITOR_NS, name.as_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub handle: WindowHandle,
    pub title: String,
    pub rect: Rect,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowHandle(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
        // dpi_scale cancels for the position term (self.x * monitor.width)
        // because X11 reports physical-pixel dimensions for both monitors
        // and cursor coordinates. Scale is applied to gap/margin for high-DPI.
        let scale = monitor.dpi_scale.max(1.0);
        let mx = monitor.x as f64 + self.x * monitor.width as f64 + self.margin as f64 * scale + (self.gap as f64 * scale / 2.0);
        let my = monitor.y as f64 + self.y * monitor.height as f64 + self.margin as f64 * scale + (self.gap as f64 * scale / 2.0);
        let mw = (self.width * monitor.width as f64) - 2.0 * self.margin as f64 * scale - self.gap as f64 * scale;
        let mh = (self.height * monitor.height as f64) - 2.0 * self.margin as f64 * scale - self.gap as f64 * scale;
        Rect {
            x: mx.floor() as i32,
            y: my.floor() as i32,
            width: (mw.floor() as u32).max(1),
            height: (mh.floor() as u32).max(1),
        }
    }

    pub fn contains(&self, px: f64, py: f64, monitor: &Monitor) -> bool {
        // dpi_scale mathematically cancels in all comparisons because X11
        // uses physical pixels for both monitor dimensions and cursor
        // coordinates. It is included for consistency with the spec.
        let scale = monitor.dpi_scale.max(1.0);
        let ex = self.x * monitor.width as f64 * scale;
        let ey = self.y * monitor.height as f64 * scale;
        let ew = self.width * monitor.width as f64 * scale;
        let eh = self.height * monitor.height as f64 * scale;
        px * scale >= ex && px * scale <= ex + ew && py * scale >= ey && py * scale <= ey + eh
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
    pub default_layout_id: Option<uuid::Uuid>,
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
            default_layout_id: None,
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
