use std::sync::{mpsc, Arc, Mutex};

use super::PlatformApi;
use crate::types::*;

pub struct MockPlatformApi {
    pub monitors: Arc<Mutex<Vec<Monitor>>>,
    pub windows: Arc<Mutex<Vec<Window>>>,
    pub cursor_pos: Arc<Mutex<(i32, i32)>>,
    pub mouse_down: Arc<Mutex<bool>>,
    pub move_events_tx: mpsc::Sender<WindowMoveEvent>,
    move_rx: Mutex<Option<mpsc::Receiver<WindowMoveEvent>>>,
    pub display_events_tx: mpsc::Sender<DisplayChangeEvent>,
    display_rx: Mutex<Option<mpsc::Receiver<DisplayChangeEvent>>>,
    pub moved_windows: Arc<Mutex<Vec<(WindowHandle, Rect)>>>,
}

impl MockPlatformApi {
    pub fn new() -> Self {
        let (move_tx, move_rx) = mpsc::channel();
        let (display_tx, display_rx) = mpsc::channel();
        Self {
            monitors: Arc::new(Mutex::new(vec![])),
            windows: Arc::new(Mutex::new(vec![])),
            cursor_pos: Arc::new(Mutex::new((0, 0))),
            mouse_down: Arc::new(Mutex::new(false)),
            move_events_tx: move_tx,
            move_rx: Mutex::new(Some(move_rx)),
            display_events_tx: display_tx,
            display_rx: Mutex::new(Some(display_rx)),
            moved_windows: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn add_monitor(&self, monitor: Monitor) {
        self.monitors.lock().unwrap().push(monitor);
    }

    pub fn set_cursor(&self, x: i32, y: i32) {
        *self.cursor_pos.lock().unwrap() = (x, y);
    }

    pub fn set_mouse_down(&self, down: bool) {
        *self.mouse_down.lock().unwrap() = down;
    }

    pub fn send_move_event(&self, event: WindowMoveEvent) {
        let _ = self.move_events_tx.send(event);
    }

    pub fn send_display_event(&self, event: DisplayChangeEvent) {
        let _ = self.display_events_tx.send(event);
    }

    pub fn get_moved_windows(&self) -> Vec<(WindowHandle, Rect)> {
        self.moved_windows.lock().unwrap().clone()
    }
}

impl PlatformApi for MockPlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor> {
        self.monitors.lock().unwrap().clone()
    }

    fn enumerate_windows(&self) -> Vec<Window> {
        self.windows.lock().unwrap().clone()
    }

    fn move_window(&self, handle: WindowHandle, rect: Rect) {
        self.moved_windows.lock().unwrap().push((handle, rect));
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        *self.cursor_pos.lock().unwrap()
    }

    fn is_mouse_button_down(&self) -> bool {
        *self.mouse_down.lock().unwrap()
    }

    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
        self.move_rx
            .lock()
            .unwrap()
            .take()
            .expect("subscribe_window_move_events called more than once")
    }

    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
        self.display_rx
            .lock()
            .unwrap()
            .take()
            .expect("subscribe_display_change_events called more than once")
    }

    fn create_overlay_window(&self, _monitor_id: MonitorId) -> Result<OverlayHandle, String> {
        Ok(OverlayHandle(1))
    }

    fn overlay_present(&self, _handle: &OverlayHandle, _pixels: &[u8], _w: u32, _h: u32) {}

    fn destroy_overlay_window(&self, _handle: OverlayHandle) {}

    fn set_autostart(&self, _enabled: bool) -> Result<(), String> {
        Ok(())
    }
}
