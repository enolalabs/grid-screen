use crate::platform_adapter::PlatformAdapter;
use crate::config_store::ConfigStore;
use crate::layout_engine::LayoutEngine;
use crate::arrange_orchestrator::ArrangeOrchestrator;
use crate::diagnostics::Diagnostics;
use crate::window_catalog;
use shared_types::*;
use tauri::State;

pub struct AppState {
    pub adapter: Box<dyn PlatformAdapter>,
    pub config: ConfigStore,
}

#[tauri::command]
pub fn bootstrap(state: State<AppState>) -> Result<BootstrapData, String> {
    let (settings, layouts, _warnings) = state.config.load()?;
    let screens = state.adapter.enumerate_screens();

    let workspace = state.adapter.current_workspace();
    let windows = state.adapter.enumerate_windows(&workspace);
    let eligible: Vec<_> = windows
        .into_iter()
        .filter(|w| window_catalog::is_eligible_window(w))
        .collect();

    let system_status = state.adapter.detect_capabilities();

    Ok(BootstrapData {
        screens,
        layouts,
        windows: eligible,
        settings,
        system_status,
    })
}

#[tauri::command]
pub fn refresh_windows(state: State<AppState>) -> Vec<WindowDescriptor> {
    let workspace = state.adapter.current_workspace();
    let windows = state.adapter.enumerate_windows(&workspace);
    windows
        .into_iter()
        .filter(|w| window_catalog::is_eligible_window(w))
        .collect()
}

#[tauri::command]
pub fn arrange_windows(state: State<AppState>, request: ArrangeRequest) -> ArrangeResult {
    let (_, layouts, _) = state
        .config
        .load()
        .unwrap_or_else(|_| (Settings::default(), Vec::new(), Vec::new()));
    let screens = state.adapter.enumerate_screens();

    ArrangeOrchestrator::arrange(
        &request,
        &layouts,
        &screens,
        state.adapter.as_ref(),
        &LayoutEngine,
    )
}

#[tauri::command]
pub fn save_layout(state: State<AppState>, layout: Layout) -> Result<(), String> {
    let (_, mut layouts, _) = state.config.load()?;
    if let Some(existing) = layouts.iter_mut().find(|l| l.id == layout.id) {
        *existing = layout;
    } else {
        layouts.push(layout);
    }
    state.config.save_layouts(&layouts)
}

#[tauri::command]
pub fn delete_layout(state: State<AppState>, layout_id: String) -> Result<(), String> {
    let (_, mut layouts, _) = state.config.load()?;
    layouts.retain(|l| l.id != layout_id);
    state.config.save_layouts(&layouts)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state
        .config
        .load()
        .map(|(s, _, _)| s)
        .unwrap_or_default()
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, settings: Settings) -> Result<(), String> {
    state.config.save_settings(&settings)
}

#[tauri::command]
pub fn save_defaults(state: State<AppState>, gap_px: u32, margin_px: u32) -> Result<(), String> {
    state.config.save_defaults(gap_px, margin_px)
}

#[tauri::command]
pub fn get_diagnostics(state: State<AppState>) -> String {
    let status = state.adapter.detect_capabilities();
    Diagnostics::collect_info(&status)
}
