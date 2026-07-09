use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use tracing;
use uuid::Uuid;

use crate::config_store::ConfigStore;
use crate::types::*;

pub struct LayoutManager;

impl LayoutManager {
    pub fn get_zones(monitor: &Monitor, active_layouts: &ArcSwap<Vec<Layout>>) -> Vec<Zone> {
        match active_layouts.load().iter().find(|l| l.monitor_id == monitor.id) {
            Some(layout) => layout.zones.clone(),
            None => Self::default_layout_for(monitor).zones,
        }
    }

    pub fn activate(layout: Layout, active_layouts: &Arc<ArcSwap<Vec<Layout>>>) {
        let mut layouts = active_layouts.load().to_vec();
        layouts.retain(|l| l.monitor_id != layout.monitor_id);
        layouts.push(layout);
        active_layouts.store(Arc::new(layouts));
    }

    pub fn save_layout(
        name: &str, zones: Vec<Zone>, monitor_id: MonitorId, arrangement_id: &str,
        config_store: &ConfigStore, saved_layouts: &RwLock<Vec<SavedLayout>>,
    ) -> Result<Uuid, String> {
        let mut config = config_store.load();
        let id = Uuid::new_v4();
        config.layouts.push(SavedLayout {
            id, name: name.trim().to_string(), arrangement_id: arrangement_id.to_string(),
            zones, monitor_id,
        });
        config_store.save(&config).map_err(|e| e.to_string())?;
        *saved_layouts.write().unwrap() = config.layouts.clone();
        Ok(id)
    }

    pub fn list_layouts(config_store: &ConfigStore) -> Vec<SavedLayout> {
        config_store.load().layouts
    }

    pub fn delete_layout(id: Uuid, config_store: &ConfigStore, saved_layouts: &RwLock<Vec<SavedLayout>>) -> Result<(), String> {
        let mut config = config_store.load();
        config.layouts.retain(|l| l.id != id);
        config_store.save(&config).map_err(|e| e.to_string())?;
        *saved_layouts.write().unwrap() = config.layouts.clone();
        Ok(())
    }

    pub fn default_layout_for(monitor: &Monitor) -> Layout {
        Layout {
            zones: vec![Zone {
                id: Uuid::new_v4(), name: "Full Screen".into(),
                x: 0.0, y: 0.0, width: 1.0, height: 1.0, gap: 0, margin: 0,
            }],
            monitor_id: monitor.id,
        }
    }
}
