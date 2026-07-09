use std::sync::Arc;

use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};
use tracing;

use crate::platform::PlatformApi;
use crate::types::*;

pub struct ZoneOverlay {
    api: Arc<dyn PlatformApi>,
    active_overlay: Option<OverlayHandle>,
    current_monitor: Option<Monitor>,
    pixmap: Option<Pixmap>,
    prev_highlighted_zone_id: Option<uuid::Uuid>,
    prev_ghost_rect: Option<Rect>,
}

impl ZoneOverlay {
    pub fn new(api: Arc<dyn PlatformApi>) -> Self {
        Self {
            api,
            active_overlay: None,
            current_monitor: None,
            pixmap: None,
            prev_highlighted_zone_id: None,
            prev_ghost_rect: None,
        }
    }

    pub fn show(&mut self, monitor: Monitor) {
        if self.active_overlay.is_some() {
            self.hide();
        }
        let w = monitor.width;
        let h = monitor.height;
        self.pixmap = Pixmap::new(w, h);
        match self.api.create_overlay_window(monitor.id) {
            Ok(handle) => {
                self.active_overlay = Some(handle);
                self.current_monitor = Some(monitor);
            }
            Err(e) => {
                tracing::warn!("Failed to create overlay window: {:?}", e);
                self.pixmap = None;
            }
        }
    }

    pub fn update(&mut self, highlighted_zone: Option<&Zone>, ghost_rect: Option<Rect>, monitor: &Monitor) {
        let handle = match &self.active_overlay {
            Some(h) => h,
            None => return,
        };

        let pixmap = match &mut self.pixmap {
            Some(p) => p,
            None => return,
        };

        let zone_changed = highlighted_zone.map(|z| z.id) != self.prev_highlighted_zone_id;
        let ghost_changed = ghost_rect != self.prev_ghost_rect;

        if !zone_changed && !ghost_changed {
            return;
        }

        self.prev_highlighted_zone_id = highlighted_zone.map(|z| z.id);
        self.prev_ghost_rect = ghost_rect;

        // Clear and reuse pre-allocated pixmap
        pixmap.fill_path(
            &PathBuilder::from_rect(Rect::from_xywh(0.0, 0.0, monitor.width as f32, monitor.height as f32).unwrap()),
            &Paint::default(),
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );

        if let Some(zone) = highlighted_zone {
            let mut paint = Paint::default();
            paint.set_color_rgba8(124, 58, 237, 51);
            let rect = zone.effective_rect(monitor);
            let path = PathBuilder::from_rect(Rect::from_xywh(
                rect.x as f32, rect.y as f32,
                rect.width as f32, rect.height as f32,
            ).unwrap());
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }

        if let Some(rect) = ghost_rect {
            let mut paint = Paint::default();
            paint.set_color_rgba8(124, 58, 237, 128);
            let path = PathBuilder::from_rect(Rect::from_xywh(
                rect.x as f32, rect.y as f32,
                rect.width as f32, rect.height as f32,
            ).unwrap());
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }

        // NOTE: tiny-skia Pixmap data format is platform-dependent.
        // On X11: native-endian ARGB. On Windows: premultiplied BGRA.
        // Pixel format conversion may be needed before overlay_present.
        self.api.overlay_present(handle, pixmap.data(), monitor.width, monitor.height);
        tracing::trace!("Overlay frame presented {}x{}", monitor.width, monitor.height);
    }

    pub fn hide(&mut self) {
        if let Some(handle) = self.active_overlay.take() {
            self.api.destroy_overlay_window(handle);
        }
        self.current_monitor = None;
        self.pixmap = None;
        self.prev_highlighted_zone_id = None;
        self.prev_ghost_rect = None;
    }
}
