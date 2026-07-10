use std::sync::Arc;
use std::thread;
use std::time::Duration;

use arc_swap::ArcSwap;
use tracing;

use crate::platform::PlatformApi;
use crate::types::*;

pub struct MonitorManager {
    monitors: Arc<ArcSwap<Vec<Monitor>>>,
}

impl MonitorManager {
    pub fn new(api: Arc<dyn PlatformApi>) -> Self {
        let monitors = Arc::new(ArcSwap::from_pointee(api.enumerate_monitors()));
        let monitors_clone = monitors.clone();

        let api_event = Arc::clone(&api);
        thread::spawn(move || {
            let rx = api_event.subscribe_display_change_events();
            for event in rx {
                tracing::debug!("Display event: {:?}", event);
                let updated = api_event.enumerate_monitors();
                monitors_clone.store(Arc::new(updated));
            }
        });

        let monitors3 = monitors.clone();
        let api_poll = Arc::clone(&api);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(30));
            let current = api_poll.enumerate_monitors();
            if current.as_slice() != monitors3.load().as_ref().as_slice() {
                tracing::info!("Safety-net polling detected monitor change");
                monitors3.store(Arc::new(current));
            }
        });

        Self { monitors }
    }

    pub fn get_monitor_at(&self, x: i32, y: i32) -> Option<Monitor> {
        self.monitors
            .load()
            .iter()
            .find(|m| x >= m.x && x < m.x + m.width as i32 && y >= m.y && y < m.y + m.height as i32)
            .cloned()
    }

    pub fn arrangement_id(&self) -> String {
        let mons = self.monitors.load();
        let mut parts: Vec<String> = mons
            .iter()
            .map(|m| format!("{}:{}x{}@{}x{}", m.name, m.width, m.height, m.x, m.y))
            .collect();
        parts.sort();
        parts.join("|")
    }

    pub fn all_monitors(&self) -> Vec<Monitor> {
        self.monitors.load().to_vec()
    }
}
