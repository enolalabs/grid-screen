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

    assert!(temp.path().join("layouts.json").exists());
    assert!(temp.path().join("layouts.json.bak.1").exists());
    assert!(temp.path().join("layouts.json.bak.5").exists());
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

// ---- Trust-boundary negative tests ----

#[test]
fn test_validation_rejects_nan_zone_coordinates() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "NaN Zone".into(),
        x: f64::NAN, y: 0.0, width: 0.5, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "NaN Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_infinity_zone_coordinates() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Inf Zone".into(),
        x: f64::INFINITY, y: 0.0, width: 0.5, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Inf Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_zero_dimension() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Zero Width".into(),
        x: 0.0, y: 0.0, width: 0.0, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Zero Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_empty_layout_name() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Zone 1".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_oversized_layout_name() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Zone 1".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 4, margin: 8,
    };
    let name_65 = "a".repeat(65);
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: name_65,
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_whitespace_only_layout_name() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Zone 1".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "   ".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_excessive_gap() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Big Gap".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 200, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Gap Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_excessive_margin() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "Big Margin".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 4, margin: 150,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Margin Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_rejects_malformed_accent_color() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.accent_color = "not-a-color".into();

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_accepts_valid_accent_color() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.accent_color = "#7C3AED".into();

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_ok());
}

#[test]
fn test_validation_rejects_unsupported_language() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.language = "fr".into();

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_accepts_supported_language_en() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.language = "en".into();

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_ok());
}

#[test]
fn test_validation_accepts_supported_language_vi() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.language = "vi".into();

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_ok());
}

#[test]
fn test_validation_rejects_default_layout_id_referencing_nonexistent() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.default_layout_id = Some(uuid::Uuid::new_v4());

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_err());
}

#[test]
fn test_validation_accepts_null_default_layout_id() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let mut settings = AppSettings::default();
    settings.default_layout_id = None;

    let config = ConfigFile { schema_version: 1, layouts: vec![], settings };
    assert!(store.save(&config).is_ok());
}

#[test]
fn test_validation_rejects_empty_zone_name() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path().to_path_buf());

    let zone = Zone {
        id: Uuid::new_v4(), name: "".into(),
        x: 0.0, y: 0.0, width: 1.0, height: 1.0,
        gap: 4, margin: 8,
    };
    let layout = SavedLayout {
        id: Uuid::new_v4(), name: "Layout".into(),
        arrangement_id: "x".into(),
        zones: vec![zone],
        monitor_id: MonitorId(Uuid::new_v4()),
    };
    let config = ConfigFile { schema_version: 1, layouts: vec![layout], settings: AppSettings::default() };
    assert!(store.save(&config).is_err());
}
