use shared_types::*;
use crate::platform_adapter::{PlatformAdapter, WorkspaceId, EventStream};
use tokio::sync::broadcast;

pub struct X11Adapter {
    system_status: SystemStatus,
    workspace_tx: broadcast::Sender<WorkspaceChangedPayload>,
    screen_tx: broadcast::Sender<ScreenChangedPayload>,
}

impl X11Adapter {
    pub fn new() -> Result<Self, String> {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "tty".into());

        if session_type == "wayland" {
            return Err("X11 session required — Wayland detected".into());
        }

        let (workspace_tx, _) = broadcast::channel(16);
        let (screen_tx, _) = broadcast::channel(16);

        Ok(X11Adapter {
            system_status: SystemStatus {
                session_type,
                ewmh_support: "TBD — x11rb not linked".into(),
                wm_name: "unknown".into(),
                xrandr_available: false,
                workspace: "unknown".into(),
                connected_screens: "unknown".into(),
                errors: vec![],
            },
            workspace_tx,
            screen_tx,
        })
    }
}

impl PlatformAdapter for X11Adapter {
    fn enumerate_screens(&self) -> Vec<ScreenInfo> {
        vec![ScreenInfo {
            id: "DP-1".into(),
            label: "DP-1 (Primary)".into(),
            resolution: "unknown".into(),
            work_area: Rect { x: 0, y: 0, width: 1920, height: 1080 },
        }]
    }

    fn current_workspace(&self) -> WorkspaceId {
        "1".into()
    }

    fn enumerate_windows(&self, _workspace: &str) -> Vec<WindowDescriptor> {
        Vec::new()
    }

    fn get_window_state(&self, _window_id: &str) -> Option<WindowState> {
        None // stub: no real window tracking, forces stale-window detection
    }

    fn get_frame_extents(&self, _window_id: &str) -> Rect {
        Rect { x: 0, y: 0, width: 0, height: 0 }
    }

    fn restore_window(&self, _window_id: &str) {}

    fn move_resize_window(&self, _window_id: &str, _rect: Rect) -> Result<Rect, String> {
        Err("X11 move/resize not implemented in stub".into())
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
