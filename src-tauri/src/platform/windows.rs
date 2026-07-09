#[cfg(target_os = "windows")]
mod windows_impl {
    use std::sync::mpsc;
    use super::PlatformApi;
    use crate::types::*;

    pub struct WindowsPlatformApi;

    impl WindowsPlatformApi {
        pub fn new() -> Result<Self, String> {
            Ok(Self)
        }
    }

    impl PlatformApi for WindowsPlatformApi {
        fn enumerate_monitors(&self) -> Vec<Monitor> {
            // TODO: Implement via EnumDisplayMonitors + GetMonitorInfoW
            vec![]
        }

        fn enumerate_windows(&self) -> Vec<Window> {
            // TODO: Implement via EnumWindows
            vec![]
        }

        fn move_window(&self, _handle: WindowHandle, _rect: Rect) {
            // TODO: Implement via SetWindowPos
        }

        fn get_cursor_pos(&self) -> (i32, i32) {
            // TODO: Implement via GetCursorPos
            (0, 0)
        }

        fn is_mouse_button_down(&self) -> bool {
            // TODO: Implement via GetAsyncKeyState
            false
        }

        fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
            let (tx, rx) = mpsc::channel();
            std::mem::drop(tx);
            rx
        }

        fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
            let (tx, rx) = mpsc::channel();
            std::mem::drop(tx);
            rx
        }

        fn create_overlay_window(&self, _monitor_id: MonitorId) -> Result<OverlayHandle, String> {
            Err("Windows overlay not implemented".into())
        }

        fn overlay_present(&self, _handle: &OverlayHandle, _pixels: &[u8], _w: u32, _h: u32) {}

        fn destroy_overlay_window(&self, _handle: OverlayHandle) {}

        fn set_autostart(&self, _enabled: bool) -> Result<(), String> {
            // TODO: Implement via registry HKCU\Software\Microsoft\Windows\CurrentVersion\Run
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::WindowsPlatformApi;
