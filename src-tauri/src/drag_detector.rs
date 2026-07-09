use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;

use tracing;

use crate::platform::PlatformApi;
use crate::types::*;

const DRAG_THRESHOLD_PX: i32 = 5;

pub struct DragDetector {
    paused: Arc<AtomicBool>,
    stop_tx: Option<mpsc::Sender<()>>,
    drag_state: Arc<Mutex<Option<DragState>>>,
}

impl DragDetector {
    pub fn new<F1, F2>(
        api: Arc<dyn PlatformApi>,
        snap_sender: mpsc::Sender<SnapEvent>,
        mut on_show_overlay: F1,
        mut on_hide_overlay: F2,
    ) -> Self
    where
        F1: FnMut(Monitor) + Send + 'static,
        F2: FnMut() + Send + 'static,
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
                    thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }

                match rx.try_recv() {
                    Ok(event) => {
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
                                *ds = Some(DragState {
                                    window_handle: handle,
                                    original_rect: rect,
                                    snap_in_progress: false,
                                });
                                let cursor = api_drag.get_cursor_pos();
                                on_show_overlay(Monitor {
                                    id: MonitorId(uuid::Uuid::new_v4()),
                                    name: "".into(),
                                    x: 0, y: 0, width: 1920, height: 1080,
                                    dpi_scale: 1.0, is_primary: true,
                                });
                            }
                            WindowMoveEvent::DragEnd { handle, rect } => {
                                let mut ds = drag_state_clone.lock().unwrap();
                                if let Some(state) = ds.as_mut() {
                                    if state.window_handle == handle {
                                        state.snap_in_progress = true;
                                        let zone_rect = rect;
                                        let _ = snap_sender.send(SnapEvent { window_handle: handle, zone_rect });
                                        *ds = None;
                                        on_hide_overlay();
                                    }
                                }
                            }
                            WindowMoveEvent::DragMove { .. } => {}
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
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
