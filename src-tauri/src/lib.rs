pub mod app_state;
pub mod config_store;
pub mod drag_detector;
pub mod layout_manager;
pub mod monitor_manager;
pub mod platform;
pub mod types;
pub mod zone_overlay;

use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use tauri::{
    Manager, WebviewWindowBuilder,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};

use app_state::{AppConfig, AppState, FrontendState};
use config_store::ConfigStore;
use layout_manager::LayoutManager;
use types::*;

#[tauri::command]
fn get_current_state(state: tauri::State<AppState>) -> FrontendState {
    let config = state.app_config.read().unwrap();
    let monitors = state.monitors.load().to_vec();
    let active_layouts = state.active_layouts.load().to_vec();
    let saved_layouts = config.saved_layouts.read().unwrap().clone();

    FrontendState {
        monitors,
        active_layouts,
        saved_layouts,
        is_paused: config.is_paused,
        settings: config.settings.clone(),
    }
}

#[tauri::command]
fn apply_layout(
    state: tauri::State<AppState>,
    layout: Layout,
) -> Result<(), String> {
    let mut layouts = state.active_layouts.load().to_vec();
    layouts.retain(|l| l.monitor_id != layout.monitor_id);
    layouts.push(layout);
    state.active_layouts.store(Arc::new(layouts));
    Ok(())
}

#[tauri::command]
fn save_layout(
    state: tauri::State<AppState>,
    name: String,
    zones: Vec<Zone>,
    monitor_id: MonitorId,
) -> Result<(), String> {
    let config_store = ConfigStore::new(app_config_dir());
    let saved_layouts = &state.app_config.read().unwrap().saved_layouts;
    LayoutManager::save_layout(&name, zones, monitor_id, "default", &config_store, saved_layouts)?;
    Ok(())
}

#[tauri::command]
fn list_layouts(state: tauri::State<AppState>) -> Vec<SavedLayout> {
    let config_store = ConfigStore::new(app_config_dir());
    LayoutManager::list_layouts(&config_store)
}

#[tauri::command]
fn delete_layout(state: tauri::State<AppState>, id: uuid::Uuid) -> Result<(), String> {
    let config_store = ConfigStore::new(app_config_dir());
    let saved_layouts = &state.app_config.read().unwrap().saved_layouts;
    LayoutManager::delete_layout(id, &config_store, saved_layouts)?;
    Ok(())
}

#[tauri::command]
fn toggle_pause(state: tauri::State<AppState>) -> bool {
    let mut config = state.app_config.write().unwrap();
    config.is_paused = !config.is_paused;
    config.is_paused
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppSettings {
    state.app_config.read().unwrap().settings.clone()
}

#[tauri::command]
fn save_settings(state: tauri::State<AppState>, settings: AppSettings) -> Result<(), String> {
    let mut config = state.app_config.write().unwrap();
    config.settings = settings;
    Ok(())
}

fn app_config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("grid-screen")
}

pub fn run() {
    let _guard = setup_logging();

    let config_dir = app_config_dir();
    std::fs::create_dir_all(&config_dir).ok();

    let config_store = ConfigStore::new(config_dir.clone());
    let loaded_config = config_store.load();

    let app_state = AppState {
        monitors: Arc::new(ArcSwap::from_pointee(vec![])),
        active_layouts: Arc::new(ArcSwap::from_pointee(vec![])),
        drag_state: std::sync::Mutex::new(None),
        app_config: RwLock::new(AppConfig {
            is_paused: false,
            settings: loaded_config.settings,
            saved_layouts: RwLock::new(loaded_config.layouts),
        }),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_current_state,
            apply_layout,
            save_layout,
            list_layouts,
            delete_layout,
            toggle_pause,
            get_settings,
            save_settings,
        ])
        .setup(move |app| {
            let _config_window = WebviewWindowBuilder::new(
                app,
                "config-main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Grid Screen — Configuration")
            .inner_size(900.0, 650.0)
            .visible(false)
            .build()?;

            let configure = MenuItemBuilder::with_id("configure", "Configure").build(app)?;
            let pause = MenuItemBuilder::with_id("pause", "Pause").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&configure)
                .item(&pause)
                .item(&quit)
                .build()?;

            TrayIconBuilder::new("grid-screen-tray")
                .menu(&menu)
                .tooltip("Grid Screen")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "configure" => {
                            if let Some(w) = app.get_webview_window("config-main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "pause" => {
                            // Toggle handled via app state in full integration
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            tracing::info!("Grid Screen started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Grid Screen");
}

fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let config_dir = app_config_dir();
    std::fs::create_dir_all(&config_dir).ok();

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::NEVER)
        .filename_prefix("grid-screen")
        .filename_suffix("log")
        .max_file_size(1_000_000)
        .max_log_files(3)
        .build(&config_dir)
        .unwrap();

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_writer(non_blocking)
        .init();

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("PANIC: {:?}", info);
        std::process::abort();
    }));

    guard
}
