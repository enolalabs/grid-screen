use shared_types::*;
use crate::platform_adapter::{PlatformAdapter, WorkspaceId, EventStream};
use tokio::sync::broadcast;

#[derive(serde::Deserialize)]
struct RawToplevel {
    title: String,
    app_id: String,
    #[serde(default)]
    minimized: bool,
    #[serde(default)]
    maximized: bool,
    #[serde(default)]
    fullscreen: bool,
}

extern "C" {
    fn enumerate_toplevels_json() -> *mut std::os::raw::c_char;
}

pub struct WaylandAdapter {
    system_status: SystemStatus,
    toplevels: Vec<RawToplevel>,
    workspace_tx: broadcast::Sender<WorkspaceChangedPayload>,
    screen_tx: broadcast::Sender<ScreenChangedPayload>,
}

impl WaylandAdapter {
    pub fn new() -> Result<Self, String> {
        let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
        if wayland_display.is_empty() {
            return Err("No Wayland display — $WAYLAND_DISPLAY is not set".into());
        }

        let toplevels = Self::enumerate_toplevels()
            .unwrap_or_else(|e| {
                tracing::warn!("Wayland enumeration failed: {}", e);
                Vec::new()
            });

        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "wayland".into());
        let wm_name = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into());
        let xdg_session = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();

        let (workspace_tx, _) = broadcast::channel(16);
        let (screen_tx, _) = broadcast::channel(16);

        Ok(WaylandAdapter {
            system_status: SystemStatus {
                session_type,
                ewmh_support: "N/A (Wayland)".into(),
                wm_name: format!("{} ({})", wm_name, xdg_session),
                xrandr_available: false,
                workspace: "1".into(),
                connected_screens: "Wayland outputs".into(),
                errors: vec!["Window move/resize not supported on Wayland — arrangement disabled".into()],
            },
            toplevels,
            workspace_tx,
            screen_tx,
        })
    }

    fn enumerate_toplevels() -> Result<Vec<RawToplevel>, String> {
        // SAFETY: C FFI call to wayland helper
        unsafe {
            let ptr = enumerate_toplevels_json();
            if ptr.is_null() {
                return Ok(Vec::new());
            }
            let json = std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string();
            serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse Wayland JSON: {} — raw: {}", e, &json[..200.min(json.len())]))
        }
    }

    fn derive_icon_color(app_id: &str) -> String {
        let mut hash: u32 = 0;
        for b in app_id.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u32);
        }
        let hue = hash % 360;
        format!("hsl({}, 55%, 50%)", hue)
    }
}

impl PlatformAdapter for WaylandAdapter {
    fn enumerate_screens(&self) -> Vec<ScreenInfo> {
        vec![ScreenInfo {
            id: "wayland-0".into(),
            label: "Wayland Display".into(),
            resolution: "unknown".into(),
            work_area: Rect { x: 0, y: 0, width: 1920, height: 1080 },
        }]
    }

    fn current_workspace(&self) -> WorkspaceId {
        "1".into()
    }

    fn enumerate_windows(&self, _workspace: &str) -> Vec<WindowDescriptor> {
        self.toplevels.iter().enumerate().map(|(i, tl)| {
            let icon_color = Self::derive_icon_color(&tl.app_id);
            WindowDescriptor {
                id: format!("wayland-{}", i),
                app_name: if tl.app_id.is_empty() { tl.title.clone() } else { tl.app_id.clone() },
                title: tl.title.clone(),
                icon_color,
                state: WindowState {
                    minimized: tl.minimized,
                    maximized: tl.maximized,
                    fullscreen: tl.fullscreen,
                    movable: false,
                    resizable: false,
                },
            }
        }).collect()
    }

    fn get_window_state(&self, window_id: &str) -> Option<WindowState> {
        let idx: usize = window_id.strip_prefix("wayland-")?.parse().ok()?;
        self.toplevels.get(idx).map(|tl| WindowState {
            minimized: tl.minimized,
            maximized: tl.maximized,
            fullscreen: tl.fullscreen,
            movable: false,
            resizable: false,
        })
    }

    fn get_frame_extents(&self, _window_id: &str) -> Rect {
        Rect { x: 0, y: 0, width: 0, height: 0 }
    }

    fn restore_window(&self, _window_id: &str) {}

    fn move_resize_window(&self, _window_id: &str, _rect: Rect) -> Result<Rect, String> {
        Err("Window arrangement not supported on Wayland".into())
    }

    fn subscribe_workspace_events(&self) -> EventStream<WorkspaceChangedPayload> {
        self.workspace_tx.subscribe()
    }

    fn subscribe_screen_events(&self) -> EventStream<ScreenChangedPayload> {
        self.screen_tx.subscribe()
    }

    fn detect_capabilities(&self) -> SystemStatus {
        self.system_status.clone()
    }
}
