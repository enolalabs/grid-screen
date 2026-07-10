use std::fs;

#[test]
fn test_capabilities_only_permit_expected() {
    let cap_path = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities/gridscreen.json");
    let content = fs::read_to_string(cap_path).unwrap();
    let cap: serde_json::Value = serde_json::from_str(&content).unwrap();

    let permissions: Vec<&str> = cap["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let allowed = vec![
        "core:default",
        "core:tray:default",
        "core:window:allow-close",
        "core:window:allow-set-focus",
        "core:window:allow-show",
        "core:window:allow-hide",
    ];

    for perm in &permissions {
        assert!(
            allowed.contains(perm),
            "Unexpected capability permission: {}",
            perm
        );
    }

    let forbidden = ["shell:", "http:", "fs:"];
    for perm in &permissions {
        for fb in &forbidden {
            assert!(
                !perm.starts_with(fb),
                "Forbidden capability found: {}",
                perm
            );
        }
    }
}

#[test]
fn test_csp_in_cargo_config() {
    let conf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    let content = fs::read_to_string(conf_path).unwrap();
    let conf: serde_json::Value = serde_json::from_str(&content).unwrap();

    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    assert!(csp.contains("script-src 'self'"));
    assert!(csp.contains("connect-src 'self' ipc:"));
    assert!(!csp.contains("unsafe-eval"));
}

#[test]
fn test_no_remote_permissions_in_capabilities() {
    let cap_path = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities/gridscreen.json");
    let content = fs::read_to_string(cap_path).unwrap();
    let cap: serde_json::Value = serde_json::from_str(&content).unwrap();

    let remote_keys = ["remote", "urls", "external_urls", "remote_domains"];
    for key in &remote_keys {
        assert!(
            cap.get(key).is_none(),
            "Capability contains remote permission key: {}",
            key
        );
    }
}
