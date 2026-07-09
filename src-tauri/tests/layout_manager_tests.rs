use std::sync::Arc;
use arc_swap::ArcSwap;
use std::sync::RwLock;
use grid_screen::config_store::ConfigStore;
use grid_screen::layout_manager::LayoutManager;
use grid_screen::types::*;

fn make_monitor(id: &str, w: u32, h: u32) -> Monitor {
    Monitor {
        id: MonitorId(uuid::Uuid::new_v4()), name: id.into(),
        x: 0, y: 0, width: w, height: h, dpi_scale: 1.0, is_primary: true,
    }
}

fn make_zone(name: &str, x: f64, y: f64, w: f64, h: f64) -> Zone {
    Zone { id: uuid::Uuid::new_v4(), name: name.into(), x, y, width: w, height: h, gap: 4, margin: 8 }
}

#[test]
fn test_activate_and_get_zones() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());
    let saved_layouts = RwLock::new(vec![]);
    let active_layouts = Arc::new(ArcSwap::from_pointee(vec![]));
    let monitor = make_monitor("main", 1920, 1080);

    let z1 = make_zone("left", 0.0, 0.0, 0.5, 1.0);
    let z2 = make_zone("right", 0.5, 0.0, 0.5, 1.0);

    let _id = LayoutManager::save_layout("work", vec![z1.clone(), z2.clone()], monitor.id, "test-arr", &store, &saved_layouts).unwrap();

    let layout = Layout { zones: vec![z1, z2], monitor_id: monitor.id };
    LayoutManager::activate(layout, &active_layouts);

    let zones = LayoutManager::get_zones(&monitor, &active_layouts);
    assert_eq!(zones.len(), 2);
}

#[test]
fn test_list_and_delete_layouts() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());
    let saved_layouts = RwLock::new(vec![]);
    let monitor = make_monitor("m", 1024, 768);

    let _ = LayoutManager::save_layout("alpha", vec![], monitor.id, "test-arr", &store, &saved_layouts).unwrap();
    let _ = LayoutManager::save_layout("beta", vec![], monitor.id, "test-arr", &store, &saved_layouts).unwrap();

    let list = LayoutManager::list_layouts(&store);
    assert_eq!(list.len(), 2);

    let alpha = list.iter().find(|l| l.name == "alpha").unwrap();
    LayoutManager::delete_layout(alpha.id, &store, &saved_layouts).unwrap();
    assert_eq!(LayoutManager::list_layouts(&store).len(), 1);
}

#[test]
fn test_default_layout_creates_one_zone_per_monitor() {
    let m1 = make_monitor("m1", 1920, 1080);
    let d1 = LayoutManager::default_layout_for(&m1);
    assert_eq!(d1.zones.len(), 1);
    assert_eq!(d1.zones[0].x, 0.0);
    assert_eq!(d1.zones[0].width, 1.0);
}
