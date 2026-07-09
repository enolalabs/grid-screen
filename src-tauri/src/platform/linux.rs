use std::sync::mpsc;
use std::thread;

use super::PlatformApi;
use crate::types::*;

pub struct LinuxPlatformApi;

impl LinuxPlatformApi {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

impl PlatformApi for LinuxPlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor> {
        // TODO: Implement via x11rb RandR/Xinerama
        // x11rb 0.13 has API changes — needs investigation
        vec![Monitor {
            id: MonitorId(uuid::Uuid::new_v4()),
            name: "Primary".into(),
            x: 0, y: 0,
            width: 1920, height: 1080,
            dpi_scale: 1.0,
            is_primary: true,
        }]
    }

    fn enumerate_windows(&self) -> Vec<Window> {
        vec![]
    }

    fn move_window(&self, handle: WindowHandle, rect: Rect) {
        // TODO: Implement via X11 XMoveResizeWindow
        tracing::debug!("move_window {:?} → {:?}", handle, rect);
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        // TODO: Implement via X11 XQueryPointer
        (0, 0)
    }

    fn is_mouse_button_down(&self) -> bool {
        // TODO: Implement via X11 XQueryPointer button mask
        false
    }

    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
        let (tx, rx) = mpsc::channel();
        std::mem::drop(tx);
        // TODO: Implement via X11 ConfigureNotify event loop
        // Requires proper x11rb 0.13 API adaptation
        rx
    }

    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
        let (tx, rx) = mpsc::channel();
        std::mem::drop(tx);
        rx
    }

    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String> {
        Err("X11 overlay not yet implemented".into())
    }

    fn overlay_present(&self, _handle: &OverlayHandle, _pixels: &[u8], _w: u32, _h: u32) {}

    fn destroy_overlay_window(&self, _handle: OverlayHandle) {}

    fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        // TODO: Create/remove ~/.config/autostart/grid-screen.desktop
        tracing::debug!("set_autostart: {}", enabled);
        Ok(())
    }
}
