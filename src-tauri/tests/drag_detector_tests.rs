use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;
use grid_screen::drag_detector::*;
use grid_screen::platform::mock::MockPlatformApi;
use grid_screen::platform::PlatformApi;
use grid_screen::types::*;
use uuid::Uuid;

fn make_monitor(id: &str, w: u32, h: u32) -> Monitor {
    Monitor {
        id: MonitorId(Uuid::new_v4()), name: id.into(),
        x: 0, y: 0, width: w, height: h, dpi_scale: 1.0, is_primary: true,
    }
}

#[test]
fn test_drag_detector_ignores_events_when_paused() {
    let api = Arc::new(MockPlatformApi::new());
    api.add_monitor(make_monitor("m1", 1920, 1080));
    api.set_cursor(500, 500);

    let (snap_tx, snap_rx) = mpsc::channel();
    let dt = DragDetector::new(api.clone(), snap_tx, |_| {}, || {});
    dt.set_paused(true);

    let handle = WindowHandle(42);
    api.set_mouse_down(true);
    api.send_move_event(WindowMoveEvent::DragStart { handle, rect: Rect { x: 0, y: 0, width: 800, height: 600 } });
    api.send_move_event(WindowMoveEvent::DragEnd { handle, rect: Rect { x: 500, y: 500, width: 800, height: 600 } });

    thread::sleep(Duration::from_millis(100));

    assert!(snap_rx.try_recv().is_err());
    dt.stop();
}

#[test]
fn test_snap_in_progress_blocks_repeated_detection() {
    let api = Arc::new(MockPlatformApi::new());
    let monitor = make_monitor("m1", 1920, 1080);
    api.add_monitor(monitor.clone());
    api.set_cursor(500, 500);
    api.set_mouse_down(true);

    let (snap_tx, snap_rx) = mpsc::channel();
    let dt = DragDetector::new(api.clone(), snap_tx, |_| {}, || {});

    let handle = WindowHandle(99);

    api.send_move_event(WindowMoveEvent::DragStart { handle, rect: Rect { x: 0, y: 0, width: 800, height: 600 } });
    thread::sleep(Duration::from_millis(50));

    api.send_move_event(WindowMoveEvent::DragEnd { handle, rect: Rect { x: 500, y: 500, width: 800, height: 600 } });
    thread::sleep(Duration::from_millis(50));

    api.send_move_event(WindowMoveEvent::DragStart { handle, rect: Rect { x: 500, y: 500, width: 800, height: 600 } });
    thread::sleep(Duration::from_millis(50));

    let snaps: Vec<_> = snap_rx.try_iter().collect();
    assert!(snaps.len() <= 1, "Expected ≤1 snap, got {}", snaps.len());

    dt.stop();
}
