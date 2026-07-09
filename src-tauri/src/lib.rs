pub mod app_state;
pub mod config_store;
pub mod drag_detector;
pub mod layout_manager;
pub mod monitor_manager;
pub mod perf;
pub mod platform;
pub mod types;
pub mod user_notifier;
pub mod zone_overlay;

use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use tauri::{
    Manager, WebviewWindowBuilder,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri_plugin_shell::ShellExt;

use app_state::{AppConfig, AppState, FrontendState};
use config_store::ConfigStore;
use drag_detector::DragDetector;
use layout_manager::LayoutManager;
use monitor_manager::MonitorManager;
use platform::PlatformApi;
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
    monitor_mgr: tauri::State<Arc<MonitorManager>>,
    name: String,
    zones: Vec<Zone>,
    monitor_id: MonitorId,
) -> Result<(), String> {
    let config_store = ConfigStore::new(app_config_dir());
    let saved_layouts = &state.app_config.read().unwrap().saved_layouts;
    let arrangement_id = monitor_mgr.arrangement_id();
    LayoutManager::save_layout(&name, zones, monitor_id, &arrangement_id, &config_store, saved_layouts)?;
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
fn save_settings(
    state: tauri::State<AppState>,
    platform_api: tauri::State<Arc<dyn PlatformApi>>,
    settings: AppSettings,
) -> Result<(), String> {
    let auto_start_changed = settings.auto_start != state.app_config.read().unwrap().settings.auto_start;
    let mut config = state.app_config.write().unwrap();
    config.settings = settings.clone();
    if auto_start_changed {
        platform_api.set_autostart(settings.auto_start)?;
    }
    Ok(())
}

#[tauri::command]
fn set_default_layout(
    state: tauri::State<AppState>,
    layout_id: uuid::Uuid,
) -> Result<(), String> {
    let mut config = state.app_config.write().unwrap();
    config.settings.default_layout_id = Some(layout_id);
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
            settings: loaded_config.settings.clone(),
            saved_layouts: RwLock::new(loaded_config.layouts),
        }),
    };

    // Platform API initialization
    #[cfg(target_os = "linux")]
    let platform_api: Arc<dyn PlatformApi> = {
        match platform::LinuxPlatformApi::new() {
            Ok(api) => {
                tracing::info!("X11 platform API initialized");
                Arc::new(api)
            }
            Err(e) => {
                tracing::error!("Failed to initialize X11: {}. Falling back to mock.", e);
                Arc::new(platform::mock::MockPlatformApi::new())
            }
        }
    };
    #[cfg(target_os = "windows")]
    let platform_api: Arc<dyn PlatformApi> = {
        match platform::WindowsPlatformApi::new() {
            Ok(api) => {
                tracing::info!("Windows platform API initialized");
                Arc::new(api)
            }
            Err(e) => {
                tracing::error!("Failed to init Windows platform: {}. Falling back.", e);
                Arc::new(platform::mock::MockPlatformApi::new())
            }
        }
    };

    // MonitorManager: event-driven + 30s safety-net polling
    let monitor_manager = Arc::new(MonitorManager::new(platform_api.clone()));
    let monitors_arc = app_state.monitors.clone();
    {
        let mm = monitor_manager.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            monitors_arc.store(Arc::new(mm.all_monitors()));
        });
    }

    // ZoneOverlay: shared for access from drag detector callbacks
    let overlay = Arc::new(std::sync::Mutex::new(zone_overlay::ZoneOverlay::new(platform_api.clone())));

    // DragDetector: event-driven drag processing
    let (snap_tx, snap_rx) = std::sync::mpsc::channel::<SnapEvent>();
    let drag_detector = {
        let active_layouts = app_state.active_layouts.clone();
        let overlay_show = overlay.clone();
        let overlay_update = overlay.clone();
        let overlay_hide = overlay.clone();

        Arc::new(DragDetector::new(
            platform_api.clone(),
            snap_tx,
            monitor_manager.clone(),
            active_layouts,
            move |monitor| {
                if let Ok(mut ov) = overlay_show.lock() {
                    ov.show(monitor);
                }
            },
            move |zone, ghost, monitor| {
                if let Ok(mut ov) = overlay_update.lock() {
                    ov.update(zone.as_ref(), ghost, monitor);
                }
            },
            move || {
                if let Ok(mut ov) = overlay_hide.lock() {
                    ov.hide();
                }
            },
        ))
    };

    // Snap consumer: move windows to zone rects
    let snap_api = platform_api.clone();
    std::thread::spawn(move || {
        for snap in snap_rx {
            tracing::debug!("Snapping window {:?} to {:?}", snap.window_handle, snap.zone_rect);
            snap_api.move_window(snap.window_handle, snap.zone_rect);
        }
    });

    // Load active layouts from config
    let saved_config = config_store.load();
    *app_state.app_config.write().unwrap().saved_layouts.write().unwrap() = saved_config.layouts.clone();
    for layout in &saved_config.layouts {
        let active = Layout {
            zones: layout.zones.clone(),
            monitor_id: layout.monitor_id,
        };
        let mut current = app_state.active_layouts.load().to_vec();
        current.retain(|l| l.monitor_id != active.monitor_id);
        current.push(active);
        app_state.active_layouts.store(Arc::new(current));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .manage(platform_api.clone())
        .manage(monitor_manager.clone())
        .invoke_handler(tauri::generate_handler![
            get_current_state,
            apply_layout,
            save_layout,
            list_layouts,
            delete_layout,
            toggle_pause,
            get_settings,
            save_settings,
            set_default_layout,
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
            let view_logs = MenuItemBuilder::with_id("view_logs", "View Logs").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&configure)
                .item(&pause)
                .item(&view_logs)
                .item(&quit)
                .build()?;

            let dd = drag_detector.clone();
            let dd_pause = drag_detector.clone();

            let tray = TrayIconBuilder::new()
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
                            let paused = !dd.is_paused();
                            dd.set_paused(paused);
                        }
                        "view_logs" => {
                            let log_file = dirs::config_dir()
                                .unwrap_or_default()
                                .join("grid-screen")
                                .join("grid-screen.log");
                            if log_file.exists() {
                                let _ = app.shell().open(log_file.to_string_lossy().as_ref(), None);
                            }
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
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("grid-screen")
        .filename_suffix("log")
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
