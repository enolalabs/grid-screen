use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;

use crate::types::*;

pub struct AppState {
    pub monitors: Arc<ArcSwap<Vec<Monitor>>>,
    pub active_layouts: Arc<ArcSwap<Vec<Layout>>>,
    pub drag_state: Mutex<Option<DragState>>,
    pub app_config: RwLock<AppConfig>,
}

pub struct AppConfig {
    pub is_paused: bool,
    pub settings: AppSettings,
    pub saved_layouts: RwLock<Vec<SavedLayout>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrontendState {
    pub monitors: Vec<Monitor>,
    pub active_layouts: Vec<Layout>,
    pub saved_layouts: Vec<SavedLayout>,
    pub is_paused: bool,
    pub settings: AppSettings,
}
