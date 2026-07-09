use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;

use arc_swap::ArcSwap;
use tracing;

use crate::layout_manager::LayoutManager;
use crate::monitor_manager::MonitorManager;
use crate::platform::PlatformApi;
use crate::types::*;

const DRAG_THRESHOLD_PX: i32 = 5;

pub struct DragDetector {
    paused: Arc<AtomicBool>,
    stop_tx: Option<mpsc::Sender<()>>,
    drag_state: Arc<Mutex<Option<DragState>>>,
}

impl DragDetector {
    pub fn new<F1, F2, F3>(
        api: Arc<dyn PlatformApi>,
        snap_sender: mpsc::Sender<SnapEvent>,
        monitor_manager: Arc<MonitorManager>,
        active_layouts: Arc<ArcSwap<Vec<Layout>>>,
        mut on_show_overlay: F1,
        mut on_update_overlay: F2,
        mut on_hide_overlay: F3,
    ) -> Self
    where
        F1: FnMut(Monitor) + Send + 'static,
        F2: FnMut(Option<Zone>, Option<Rect>, &Monitor) + Send + 'static,
        F3: FnMut() + Send + 'static,
    {
        let paused = Arc::new(AtomicBool::new(false));
        let drag_state = Arc::new(Mutex::new(None::<DragState>));
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        let paused_clone = paused.clone();
        let drag_state_clone = drag_state.clone();
        let api_drag = api.clone();

        thread::spawn(move || {
            let rx = api_drag.subscribe_window_move_events();

            loop {
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                if paused_clone.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }

                let event = match rx.recv() {
                    Ok(e) => e,
                    Err(_) => break,
                };

                // Clear stale snap_in_progress on any non-DragStart event
                if !matches!(event, WindowMoveEvent::DragStart { .. }) {
                    if let Ok(mut ds) = drag_state_clone.lock() {
                        if let Some(ref state) = *ds {
                            if state.snap_in_progress {
                                *ds = None;
                            }
                        }
                    }
                }

                match event {
                    WindowMoveEvent::DragStart { handle, rect } => {
                        if !api_drag.is_mouse_button_down() {
                            continue;
                        }
                        let mut ds = drag_state_clone.lock().unwrap();
                        if let Some(ref state) = *ds {
                            if state.snap_in_progress && state.window_handle == handle {
                                continue;
                            }
                        }
                        // New drag: store state, show overlay on cursor's monitor
                        *ds = Some(DragState {
                            window_handle: handle,
                            original_rect: rect,
                            snap_in_progress: false,
                        });
                        let (cx, cy) = api_drag.get_cursor_pos();
                        if let Some(monitor) = monitor_manager.get_monitor_at(cx, cy) {
                            on_show_overlay(monitor);
                        }
                    }
                    WindowMoveEvent::DragMove { handle, rect } => {
                        let ds = drag_state_clone.lock().unwrap();
                        let state = match ds.as_ref() {
                            Some(s) if s.window_handle == handle => s,
                            _ => continue,
                        };
                        let (cx, cy) = api_drag.get_cursor_pos();
                        let monitor = match monitor_manager.get_monitor_at(cx, cy) {
                            Some(m) => m,
                            None => continue,
                        };
                        let zones = LayoutManager::get_zones(&monitor, &active_layouts);
                        let mut highlighted: Option<Zone> = None;
                        for zone in &zones {
                            if zone.contains(cx as f64 - monitor.x as f64, cy as f64 - monitor.y as f64, &monitor) {
                                highlighted = Some(zone.clone());
                                break;
                            }
                        }

                        let ghost = highlighted.as_ref().map(|z| z.effective_rect(&monitor))
                            .unwrap_or(Rect { x: cx - rect.width as i32 / 2, y: cy - rect.height as i32 / 2, width: rect.width, height: rect.height });

                        on_update_overlay(highlighted, Some(ghost), &monitor);
                    }
                    WindowMoveEvent::DragEnd { handle, rect } => {
                        let mut ds = drag_state_clone.lock().unwrap();
                        let state = match ds.as_mut() {
                            Some(s) if s.window_handle == handle => s,
                            _ => continue,
                        };
                        let (cx, cy) = api_drag.get_cursor_pos();
                        let monitor = match monitor_manager.get_monitor_at(cx, cy) {
                            Some(m) => m,
                            None => {
                                *ds = None;
                                on_hide_overlay();
                                continue;
                            }
                        };
                        let zones = LayoutManager::get_zones(&monitor, &active_layouts);
                        let hit_zone = zones.iter().find(|z| {
                            z.contains(cx as f64 - monitor.x as f64, cy as f64 - monitor.y as f64, &monitor)
                        });

                        if let Some(zone) = hit_zone {
                            state.snap_in_progress = true;
                            let zone_rect = zone.effective_rect(&monitor);
                            let _ = snap_sender.send(SnapEvent { window_handle: handle, zone_rect });
                        }
                        // Keep drag_state with snap_in_progress flag to block repeated detection
                        // Clear on next idle (non-DragStart event above)
                        on_hide_overlay();
                    }
                }
            }
            tracing::info!("DragDetector event loop stopped");
        });

        Self {
            paused,
            stop_tx: Some(stop_tx),
            drag_state,
        }
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
        if paused {
            if let Ok(mut ds) = self.drag_state.lock() {
                *ds = None;
            }
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn stop(self) {
        if let Some(tx) = self.stop_tx {
            let _ = tx.send(());
        }
    }
}
