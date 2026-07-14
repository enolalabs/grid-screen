use shared_types::*;

pub type WorkspaceId = String;
pub type EventStream<T> = tokio::sync::broadcast::Receiver<T>;

pub trait PlatformAdapter: Send + Sync {
    fn enumerate_screens(&self) -> Vec<ScreenInfo>;
    fn current_workspace(&self) -> WorkspaceId;
    fn enumerate_windows(&self, workspace: &WorkspaceId) -> Vec<WindowDescriptor>;
    fn get_window_state(&self, window_id: &str) -> Option<WindowState>;
    fn get_frame_extents(&self, window_id: &str) -> Rect;
    fn restore_window(&self, window_id: &str);
    fn move_resize_window(&self, window_id: &str, rect: Rect) -> Result<Rect, String>;
    fn subscribe_workspace_events(&self) -> EventStream<WorkspaceChangedPayload>;
    fn subscribe_screen_events(&self) -> EventStream<ScreenChangedPayload>;
    fn detect_capabilities(&self) -> SystemStatus;
}

/// Test-only mock. Holds fake screens, windows, and a registry
/// of move/resize results for verification.
pub struct MockPlatformAdapter {
    pub screens: Vec<ScreenInfo>,
    pub windows: Vec<WindowDescriptor>,
    pub workspace: String,
    pub system_status: SystemStatus,
    pub move_log: std::sync::Mutex<Vec<(String, Rect)>>,
    pub workspace_tx: tokio::sync::broadcast::Sender<WorkspaceChangedPayload>,
    pub screen_tx: tokio::sync::broadcast::Sender<ScreenChangedPayload>,
    pub frame_extents: Rect,
}

impl MockPlatformAdapter {
    pub fn new() -> Self {
        let (workspace_tx, _) = tokio::sync::broadcast::channel(16);
        let (screen_tx, _) = tokio::sync::broadcast::channel(16);
        MockPlatformAdapter {
            screens: vec![
                ScreenInfo {
                    id: "DP-1".into(),
                    label: "DP-1 (Primary)".into(),
                    resolution: "2560 x 1440".into(),
                    work_area: Rect { x: 0, y: 0, width: 2560, height: 1400 },
                },
            ],
            windows: vec![],
            workspace: "1".into(),
            system_status: SystemStatus {
                session_type: "x11".into(),
                ewmh_support: "Full".into(),
                wm_name: "MockWM".into(),
                xrandr_available: true,
                workspace: "1".into(),
                connected_screens: "DP-1".into(),
                errors: vec![],
            },
            move_log: std::sync::Mutex::new(Vec::new()),
            workspace_tx,
            screen_tx,
            frame_extents: Rect { x: 0, y: 0, width: 0, height: 0 },
        }
    }
}

impl PlatformAdapter for MockPlatformAdapter {
    fn enumerate_screens(&self) -> Vec<ScreenInfo> {
        self.screens.clone()
    }

    fn current_workspace(&self) -> WorkspaceId {
        self.workspace.clone()
    }

    fn enumerate_windows(&self, _workspace: &WorkspaceId) -> Vec<WindowDescriptor> {
        self.windows.clone()
    }

    fn get_window_state(&self, window_id: &str) -> Option<WindowState> {
        self.windows.iter().find(|w| w.id == window_id).map(|w| w.state.clone())
    }

    fn get_frame_extents(&self, _window_id: &str) -> Rect {
        self.frame_extents.clone()
    }

    fn restore_window(&self, _window_id: &str) {}

    fn move_resize_window(&self, window_id: &str, rect: Rect) -> Result<Rect, String> {
        self.move_log.lock().unwrap().push((window_id.to_string(), rect.clone()));
        Ok(rect)
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
