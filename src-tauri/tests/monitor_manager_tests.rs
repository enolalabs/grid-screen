use std::sync::Arc;
use grid_screen::monitor_manager::MonitorManager;
use grid_screen::platform::mock::MockPlatformApi;
use grid_screen::platform::PlatformApi;
use grid_screen::types::*;

fn make_monitor(id: &str, x: i32, y: i32, w: u32, h: u32, primary: bool) -> Monitor {
    Monitor {
        id: MonitorId(uuid::Uuid::new_v4()),
        name: id.into(),
        x, y, width: w, height: h,
        dpi_scale: 1.0,
        is_primary: primary,
    }
}

#[test]
fn test_monitor_at_position() {
    let api = Arc::new(MockPlatformApi::new());
    api.add_monitor(make_monitor("m1", 0, 0, 1920, 1080, true));
    api.add_monitor(make_monitor("m2", 1920, 0, 1920, 1080, false));

    let mgr = MonitorManager::new(api);
    assert_eq!(mgr.get_monitor_at(100, 100).unwrap().name, "m1");
    assert_eq!(mgr.get_monitor_at(2000, 100).unwrap().name, "m2");
    assert!(mgr.get_monitor_at(-10, 0).is_none());
}

#[test]
fn test_arrangement_id_changes_on_hotplug() {
    let api = Arc::new(MockPlatformApi::new());
    api.add_monitor(make_monitor("m1", 0, 0, 1920, 1080, true));

    let mgr = MonitorManager::new(api.clone());
    let id1 = mgr.arrangement_id();

    api.add_monitor(make_monitor("m2", 1920, 0, 1920, 1080, false));
    api.send_display_event(DisplayChangeEvent::Connected);

    let id2 = mgr.arrangement_id();
    assert_ne!(id1, id2);
}
