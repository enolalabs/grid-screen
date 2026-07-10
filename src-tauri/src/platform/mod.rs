pub mod mock;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub mod hyprland;

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatformApi;

#[cfg(target_os = "linux")]
pub use hyprland::HyprlandPlatformApi;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatformApi;

use std::sync::mpsc;
use crate::types::*;

pub trait PlatformApi: Send + Sync {
    fn enumerate_monitors(&self) -> Vec<Monitor>;
    fn enumerate_windows(&self) -> Vec<Window>;
    fn move_window(&self, handle: WindowHandle, rect: Rect);
    fn get_cursor_pos(&self) -> (i32, i32);
    fn is_mouse_button_down(&self) -> bool;
    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent>;
    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent>;
    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String>;
    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32);
    fn destroy_overlay_window(&self, handle: OverlayHandle);
    fn set_autostart(&self, enabled: bool) -> Result<(), String>;
}

/// Create the best platform API for the current session.
/// On Linux, prefers HyprlandPlatformApi when running under Hyprland,
/// falls back to LinuxPlatformApi (X11).
#[cfg(target_os = "linux")]
pub fn create_platform_api() -> Arc<dyn PlatformApi> {
    if hyprland::is_hyprland_session() {
        match HyprlandPlatformApi::new() {
            Ok(api) => {
                tracing::info!("Hyprland platform API initialized");
                return Arc::new(api);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize Hyprland API ({}), falling back to X11", e);
            }
        }
    }
    match LinuxPlatformApi::new() {
        Ok(api) => {
            tracing::info!("X11 platform API initialized");
            Arc::new(api)
        }
        Err(e) => {
            tracing::error!("Failed to initialize X11: {}. Falling back to mock.", e);
            Arc::new(mock::MockPlatformApi::new())
        }
    }
}

/// Create the best platform API for the current session (Windows).
#[cfg(target_os = "windows")]
pub fn create_platform_api() -> Arc<dyn PlatformApi> {
    match WindowsPlatformApi::new() {
        Ok(api) => {
            tracing::info!("Windows platform API initialized");
            Arc::new(api)
        }
        Err(e) => {
            tracing::error!("Failed to init Windows platform: {}. Falling back.", e);
            Arc::new(mock::MockPlatformApi::new())
        }
    }
}

use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use mock::MockPlatformApi;
    use crate::types::*;

    #[test]
    fn test_zone_effective_rect() {
        let monitor = Monitor {
            id: MonitorId(uuid::Uuid::new_v4()),
            name: "test".into(),
            x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1.0, is_primary: true,
        };
        let zone = Zone {
            id: uuid::Uuid::new_v4(),
            name: "left-half".into(),
            x: 0.0, y: 0.0, width: 0.5, height: 1.0,
            gap: 10, margin: 8,
        };
        let rect = zone.effective_rect(&monitor);
        assert_eq!(rect.x, 13);
        assert_eq!(rect.y, 13);
        assert_eq!(rect.width, 925);
    }

    #[test]
    fn test_zone_contains() {
        let monitor = Monitor {
            id: MonitorId(uuid::Uuid::new_v4()),
            name: "test".into(),
            x: 0, y: 0, width: 1000, height: 1000, dpi_scale: 1.0, is_primary: true,
        };
        let zone = Zone {
            id: uuid::Uuid::new_v4(),
            name: "center".into(),
            x: 0.25, y: 0.25, width: 0.5, height: 0.5,
            gap: 0, margin: 0,
        };
        let px = 500.0;
        let py = 500.0;
        assert!(zone.contains(px, py, &monitor));
        assert!(!zone.contains(100.0, 100.0, &monitor));
    }
}
