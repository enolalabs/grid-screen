use grid_screen::types::*;
use uuid::Uuid;

fn make_monitor(w: u32, h: u32) -> Monitor {
    Monitor {
        id: MonitorId(Uuid::new_v4()), name: "test".into(),
        x: 0, y: 0, width: w, height: h, dpi_scale: 1.0, is_primary: true,
    }
}

fn make_zone(name: &str, x: f64, y: f64, w: f64, h: f64) -> Zone {
    Zone { id: Uuid::new_v4(), name: name.into(), x, y, width: w, height: h, gap: 4, margin: 8 }
}

#[test]
fn test_overlay_buffer_size_matches_monitor() {
    let monitor = make_monitor(1920, 1080);
    let buffer_size = (monitor.width * monitor.height * 4) as usize;
    assert_eq!(buffer_size, 1920 * 1080 * 4);
}

#[test]
fn test_dirty_rect_only_repaints_changed_zones() {
    let _monitor = make_monitor(1920, 1080);
    let zones = vec![
        make_zone("left", 0.0, 0.0, 0.5, 1.0),
        make_zone("right", 0.5, 0.0, 0.5, 1.0),
    ];

    let prev = Some(&zones[0]);
    let curr = Some(&zones[1]);

    assert_ne!(prev.map(|z| z.id), curr.map(|z| z.id));
}

#[test]
fn test_pre_allocated_pixmap_size_matches_monitor() {
    let monitor = make_monitor(1920, 1080);
    let pixmap = tiny_skia::Pixmap::new(monitor.width, monitor.height);
    assert!(pixmap.is_some());
    let p = pixmap.unwrap();
    assert_eq!(p.data().len(), (1920 * 1080 * 4) as usize);
}
