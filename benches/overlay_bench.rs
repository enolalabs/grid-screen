use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grid_screen::types::*;

fn bench_zone_hit_test(c: &mut Criterion) {
    let monitor = Monitor {
        id: MonitorId(uuid::Uuid::new_v4()), name: "4k".into(),
        x: 0, y: 0, width: 3840, height: 2160, dpi_scale: 1.5, is_primary: true,
    };
    let zones: Vec<Zone> = (0..64).map(|i| Zone {
        id: uuid::Uuid::new_v4(), name: format!("z{}", i),
        x: (i as f64 % 8.0) / 8.0, y: (i as f64 / 8.0).floor() / 8.0,
        width: 1.0 / 8.0, height: 1.0 / 8.0,
        gap: 4, margin: 8,
    }).collect();

    c.bench_function("hit_test_64_zones", |b| {
        b.iter(|| {
            for zone in &zones {
                black_box(zone.contains(1920.0, 1080.0, &monitor));
            }
        });
    });
}

criterion_group!(benches, bench_zone_hit_test);
criterion_main!(benches);
