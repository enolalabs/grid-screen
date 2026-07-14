#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dependency_on_unit_never_type_fallback)]

#[macro_use]
mod app_shell;
mod platform_adapter;
mod x11_adapter;
mod config_store;
mod layout_engine;
mod window_catalog;
mod arrange_orchestrator;
mod diagnostics;

use app_shell::*;
use config_store::ConfigStore;
use platform_adapter::{MockPlatformAdapter, PlatformAdapter};
use x11_adapter::X11Adapter;
use diagnostics::Diagnostics;
use std::path::PathBuf;
use tauri::Manager;

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config")
        })
        .join("grid-screen")
}

fn main() {
    let config_path = config_dir();
    std::fs::create_dir_all(&config_path).unwrap();
    Diagnostics::init(&config_path);

    let config = ConfigStore::new(config_path.clone());
    config.seed_presets().expect("Failed to seed layout presets");

    tracing::info!("Grid Screen starting, config dir: {:?}", config_path);

    let adapter: Box<dyn PlatformAdapter> = match X11Adapter::new() {
        Ok(x11) => Box::new(x11),
        Err(e) => {
            tracing::warn!("X11 adapter failed: {} — using mock", e);
            let mut mock = MockPlatformAdapter::new();
            mock.system_status.errors.push(e);
            Box::new(mock)
        }
    };

    let app_state = AppState {
        adapter,
        config,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            refresh_windows,
            arrange_windows,
            save_layout,
            delete_layout,
            get_settings,
            update_settings,
            save_defaults,
            get_diagnostics,
        ])
        .setup(|_app| {
            tracing::info!("Application setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let settings = state.config.load().map(|(s, _, _)| s).unwrap_or_default();
                if settings.minimize_to_tray {
                    api.prevent_close();
                    window.hide().ok();
                    tracing::info!("Window minimized to tray");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    tracing::info!("Grid Screen exited");
}
