use grid_screen::config_store::ConfigStore;
use grid_screen::types::*;
use uuid::Uuid;

#[test]
fn test_save_and_load_config() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Zone 1".into(),
        x: 0.0, y: 0.0, width: 0.5, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "My Layout".into(),
        arrangement_id: "abc123".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile {
        schema_version: 1,
        layouts: vec![layout],
        settings: AppSettings::default(),
    };

    store.save(&config).unwrap();
    let loaded = store.load();
    assert_eq!(loaded.schema_version, 1);
    assert_eq!(loaded.layouts.len(), 1);
    assert_eq!(loaded.layouts[0].name, "My Layout");
}

#[test]
fn test_load_corrupted_config_falls_back_to_default() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("layouts.json");
    std::fs::write(&config_path, b"not valid json").unwrap();

    let store = ConfigStore::new(temp.path().to_path_buf());
    let loaded = store.load();
    assert_eq!(loaded.schema_version, 1);
    assert!(loaded.layouts.is_empty());
}

#[test]
fn test_write_creates_backups() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    for i in 0..7 {
        let mut config = ConfigFile::default();
        config.settings.default_gap = i;
        store.save(&config).unwrap();
    }

    // Should have 5 backup files (.bak.1 through .bak.5) plus main
    assert!(temp.path().join("layouts.json").exists());
    assert!(temp.path().join("layouts.json.bak.1").exists());
    assert!(temp.path().join("layouts.json.bak.5").exists());
    // .bak.6 should NOT exist (max 5)
    assert!(!temp.path().join("layouts.json.bak.6").exists());
}

#[test]
fn test_validation_rejects_negative_coordinates() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Bad".into(),
        x: -0.1, y: 0.0, width: 0.5, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Bad Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };

    let result = store.save(&config);
    assert!(result.is_err());
}

#[test]
fn test_validation_rejects_zone_overlap() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let z1 = Zone { id: Uuid::new_v4(), name: "A".into(), x: 0.0, y: 0.0, width: 0.6, height: 1.0, gap: 0, margin: 0 };
    let z2 = Zone { id: Uuid::new_v4(), name: "B".into(), x: 0.5, y: 0.0, width: 0.6, height: 1.0, gap: 0, margin: 0 };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Overlapping".into(),
        arrangement_id: "x".into(),
        zones: vec![z1, z2],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };

    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_enforces_max_zones() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zones: Vec<Zone> = (0..65).map(|i| Zone {
        id: Uuid::new_v4(), name: format!("Z{}", i),
        x: 0.0, y: 0.0, width: 0.01, height: 0.01,
        gap: 0, margin: 0,
    }).collect();
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Too Many".into(),
        arrangement_id: "x".into(),
        zones,
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };

    assert!(store.save(&config).is_err());
}
