use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::randr::{self, ConnectionExt as _};
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xinerama::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{
    self, ChangeWindowAttributesAux, ConfigureWindowAux, CreateWindowAux, EventMask,
    KeyButMask, PropMode, VisualClass, Visualid, Window as XWindow,
};
use x11rb::protocol::xproto::ConnectionExt as XProtoExt;
use x11rb::wrapper::ConnectionExt as WrapperExt;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use super::PlatformApi;
use crate::types::*;

pub struct LinuxPlatformApi {
    conn: Arc<RustConnection>,
    screen_num: usize,
    root: XWindow,
    argb_visual: Option<Visualid>,
    argb_colormap: Option<u32>,
    overlay_gc: std::sync::Mutex<Option<xproto::Gcontext>>,
}

impl LinuxPlatformApi {
    pub fn new() -> Result<Self, String> {
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| format!("X11 connect: {}", e))?;
        let conn = Arc::new(conn);
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let (argb_visual, argb_colormap) = find_argb_visual(&conn, screen_num);

        Ok(Self {
            conn,
            screen_num,
            root,
            argb_visual,
            argb_colormap,
            overlay_gc: std::sync::Mutex::new(None),
        })
    }

    fn ensure_overlay_gc(&self, drawable: XWindow) -> xproto::Gcontext {
        let mut gc_lock = self.overlay_gc.lock().unwrap();
        if let Some(gc) = *gc_lock {
            return gc;
        }
        let gc = self
            .conn
            .generate_id()
            .ok()
            .and_then(|gc_id| {
                self.conn
                    .create_gc(gc_id, drawable, &xproto::CreateGCAux::new())
                    .ok()
                    .and_then(|c| c.check().ok())
                    .map(|_| gc_id)
            });
        if let Some(gc) = gc {
            *gc_lock = Some(gc);
            gc
        } else {
            0
        }
    }

    fn monitors_from_randr(&self) -> Option<Vec<Monitor>> {
        if !self
            .conn
            .extension_information(randr::X11_EXTENSION_NAME)
            .ok()?
            .is_some()
        {
            return None;
        }

        let resources = self
            .conn
            .randr_get_screen_resources_current(self.root)
            .ok()?
            .reply()
            .ok()?;

        let mut monitors = Vec::new();
        for &output in &resources.outputs {
            let info = match self
                .conn
                .randr_get_output_info(output, resources.config_timestamp)
                .ok()
                .and_then(|c| c.reply().ok())
            {
                Some(i) => i,
                None => continue,
            };
            if info.connection != 1.into() || info.crtc == 0 {
                continue;
            }
            let crtc = match self
                .conn
                .randr_get_crtc_info(info.crtc, resources.config_timestamp)
                .ok()
                .and_then(|c| c.reply().ok())
            {
                Some(c) => c,
                None => continue,
            };
            let name = String::from_utf8_lossy(&info.name).to_string();
            let is_primary = resources
                .outputs
                .first()
                .map(|&p| p == output)
                .unwrap_or(false);

            let (width_mm, height_mm) = (info.mm_width as f64, info.mm_height as f64);
            let dpi_scale = if width_mm > 0.0 && height_mm > 0.0 {
                let dpi = (crtc.width as f64 / width_mm * 25.4)
                    .max(crtc.height as f64 / height_mm * 25.4);
                (dpi / 96.0).clamp(1.0, 3.0)
            } else {
                1.0
            };

            monitors.push(Monitor {
                id: MonitorId(uuid::Uuid::new_v4()),
                name,
                x: crtc.x as i32,
                y: crtc.y as i32,
                width: crtc.width as u32,
                height: crtc.height as u32,
                dpi_scale,
                is_primary,
            });
        }
        Some(monitors)
    }

    fn monitors_from_xinerama(&self) -> Option<Vec<Monitor>> {
        if !self
            .conn
            .extension_information(xinerama::X11_EXTENSION_NAME)
            .ok()?
            .is_some()
        {
            return None;
        }
        let reply = self.conn.xinerama_query_screens().ok()?.reply().ok()?;
        let mut monitors = Vec::new();
        for (i, info) in reply.screen_info.iter().enumerate() {
            monitors.push(Monitor {
                id: MonitorId(uuid::Uuid::new_v4()),
                name: format!("Xinerama-{}", i),
                x: info.x_org as i32,
                y: info.y_org as i32,
                width: info.width as u32,
                height: info.height as u32,
                dpi_scale: 1.0,
                is_primary: i == 0,
            });
        }
        Some(monitors)
    }
}

impl PlatformApi for LinuxPlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor> {
        if let Some(mons) = self.monitors_from_randr() {
            if !mons.is_empty() {
                return mons;
            }
        }
        if let Some(mons) = self.monitors_from_xinerama() {
            if !mons.is_empty() {
                return mons;
            }
        }
        let screen = &self.conn.setup().roots[self.screen_num];
        vec![Monitor {
            id: MonitorId(uuid::Uuid::new_v4()),
            name: "Primary".into(),
            x: 0,
            y: 0,
            width: screen.width_in_pixels as u32,
            height: screen.height_in_pixels as u32,
            dpi_scale: 1.0,
            is_primary: true,
        }]
    }

    fn enumerate_windows(&self) -> Vec<Window> {
        let reply = match self
            .conn
            .query_tree(self.root)
            .ok()
            .and_then(|c| c.reply().ok())
        {
            Some(r) => r,
            None => return vec![],
        };
        let mut windows = Vec::new();
        for &child in &reply.children {
            let geom = match self
                .conn
                .get_geometry(child)
                .ok()
                .and_then(|c| c.reply().ok())
            {
                Some(g) => g,
                None => continue,
            };
            let attr = match self
                .conn
                .get_window_attributes(child)
                .ok()
                .and_then(|c| c.reply().ok())
            {
                Some(a) => a,
                None => continue,
            };
            if attr.map_state != xproto::MapState::VIEWABLE {
                continue;
            }
            let title = get_window_title(&self.conn, child);
            if title.is_empty() {
                continue;
            }
            let (sx, sy) = if attr.override_redirect {
                (geom.x as i32, geom.y as i32)
            } else {
                let coords = self
                    .conn
                    .translate_coordinates(child, self.root, 0, 0)
                    .ok()
                    .and_then(|c| c.reply().ok());
                match coords {
                    Some(c) => (c.dst_x as i32 - geom.x as i32, c.dst_y as i32 - geom.y as i32),
                    None => (geom.x as i32, geom.y as i32),
                }
            };
            windows.push(Window {
                handle: WindowHandle(child as u64),
                title,
                rect: Rect {
                    x: sx,
                    y: sy,
                    width: geom.width as u32,
                    height: geom.height as u32,
                },
                is_visible: true,
            });
        }
        windows
    }

    fn move_window(&self, handle: WindowHandle, rect: Rect) {
        let aux = ConfigureWindowAux::new()
            .x(rect.x)
            .y(rect.y)
            .width(rect.width as u32)
            .height(rect.height as u32);
        let _ = self.conn.configure_window(handle.0 as u32, &aux);
        let _ = self.conn.flush();
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        match self
            .conn
            .query_pointer(self.root)
            .ok()
            .and_then(|c| c.reply().ok())
        {
            Some(reply) => (reply.root_x as i32, reply.root_y as i32),
            None => (0, 0),
        }
    }

    fn is_mouse_button_down(&self) -> bool {
        match self
            .conn
            .query_pointer(self.root)
            .ok()
            .and_then(|c| c.reply().ok())
        {
            Some(reply) => reply.mask.contains(KeyButMask::BUTTON1),
            None => false,
        }
    }

    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
        let (tx, rx) = mpsc::channel();
        let conn = Arc::clone(&self.conn);
        let root = self.root;

        // Select SubstructureNotifyMask on root to get ConfigureNotify
        let aux = ChangeWindowAttributesAux::new()
            .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT);
        let _ = conn.change_window_attributes(root, &aux);
        let _ = conn.flush();

        // Polling + event-based hybrid thread
        thread::spawn(move || {
            let mut windows: HashMap<u32, Rect> = HashMap::new();
            let mut drag_handle: Option<u32> = None;
            let mut prev_mouse_down = false;

            loop {
                // Non-blocking event check
                while let Ok(Some(event)) = conn.poll_for_event() {
                    if let Event::ConfigureNotify(ev) = &event {
                        let h = ev.window;
                        let current = Rect {
                            x: ev.x as i32,
                            y: ev.y as i32,
                            width: ev.width as u32,
                            height: ev.height as u32,
                        };

                        if current.width == 0 || current.height == 0 {
                            windows.remove(&h);
                            continue;
                        }

                        let prev = windows.insert(h, current);

                        let mouse_down = conn
                            .query_pointer(root)
                            .ok()
                            .and_then(|c| c.reply().ok())
                            .map(|r| r.mask.contains(KeyButMask::BUTTON1))
                            .unwrap_or(false);

                        if let Some(prev_rect) = prev {
                            if mouse_down {
                                if prev_rect.x != current.x || prev_rect.y != current.y {
                                    if drag_handle == Some(h) {
                                        let _ = tx.send(WindowMoveEvent::DragMove {
                                            handle: WindowHandle(h as u64),
                                            rect: current,
                                        });
                                    } else {
                                        drag_handle = Some(h);
                                        let _ = tx.send(WindowMoveEvent::DragStart {
                                            handle: WindowHandle(h as u64),
                                            rect: current,
                                        });
                                    }
                                }
                            } else if let Some(dh) = drag_handle.take() {
                                let rect = windows.get(&dh).copied().unwrap_or(current);
                                let _ = tx.send(WindowMoveEvent::DragEnd {
                                    handle: WindowHandle(dh as u64),
                                    rect,
                                });
                            }
                        }
                        prev_mouse_down = mouse_down;
                    }

                    if let Event::ButtonRelease(ev) = &event {
                        if ev.detail == 1 {
                            if let Some(dh) = drag_handle.take() {
                                let rect = windows.get(&dh).copied().unwrap_or(Rect {
                                    x: 0, y: 0, width: 0, height: 0,
                                });
                                let _ = tx.send(WindowMoveEvent::DragEnd {
                                    handle: WindowHandle(dh as u64),
                                    rect,
                                });
                            }
                        }
                    }
                }

                // Polling fallback: check all windows every ~60ms
                let mouse_down = conn
                    .query_pointer(root)
                    .ok()
                    .and_then(|c| c.reply().ok())
                    .map(|r| r.mask.contains(KeyButMask::BUTTON1))
                    .unwrap_or(false);

                if mouse_down {
                    let tree = conn.query_tree(root).ok().and_then(|c| c.reply().ok());
                    if let Some(tree) = tree {
                        for &child in &tree.children {
                            let geom = conn.get_geometry(child).ok().and_then(|c| c.reply().ok());
                            if let Some(g) = geom {
                                if g.width == 0 || g.height == 0 {
                                    continue;
                                }
                                let current = Rect {
                                    x: g.x as i32,
                                    y: g.y as i32,
                                    width: g.width as u32,
                                    height: g.height as u32,
                                };
                                let prev = windows.insert(child, current);
                                if let Some(prev_rect) = prev {
                                    if (prev_rect.x != current.x || prev_rect.y != current.y) && drag_handle != Some(child) {
                                        drag_handle = Some(child);
                                        let _ = tx.send(WindowMoveEvent::DragStart {
                                            handle: WindowHandle(child as u64),
                                            rect: current,
                                        });
                                    } else if prev_rect.x != current.x || prev_rect.y != current.y {
                                        let _ = tx.send(WindowMoveEvent::DragMove {
                                            handle: WindowHandle(child as u64),
                                            rect: current,
                                        });
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(dh) = drag_handle.take() {
                    let rect = windows.get(&dh).copied().unwrap_or(Rect {
                        x: 0, y: 0, width: 0, height: 0,
                    });
                    let _ = tx.send(WindowMoveEvent::DragEnd {
                        handle: WindowHandle(dh as u64),
                        rect,
                    });
                }

                prev_mouse_down = mouse_down;
                thread::sleep(Duration::from_millis(60));
            }
        });
        rx
    }

    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
        let (tx, rx) = mpsc::channel();
        let conn = Arc::clone(&self.conn);
        let root = self.root;

        let use_randr = conn
            .extension_information(randr::X11_EXTENSION_NAME)
            .is_ok_and(|e| e.is_some());

        if use_randr {
            // RandR screen change notify mask value = 1
            let _ = conn.randr_select_input(root, 1u16.into());
            let _ = conn.flush();

            thread::spawn(move || {
                let mut last_count = 0u32;
                loop {
                    if let Some(reply) = conn
                        .randr_get_screen_resources_current(root)
                        .ok()
                        .and_then(|c| c.reply().ok())
                    {
                        let count = reply.outputs.len() as u32;
                        if last_count != 0 && count != last_count {
                            let _ = if count > last_count {
                                tx.send(DisplayChangeEvent::Connected)
                            } else {
                                tx.send(DisplayChangeEvent::Disconnected)
                            };
                        }
                        last_count = count;
                    }
                    thread::sleep(Duration::from_secs(3));
                }
            });
        } else {
            thread::spawn(move || {
                let mut last_w = 0u32;
                loop {
                    thread::sleep(Duration::from_secs(5));
                    if let Some(scr) = conn.setup().roots.first() {
                        if scr.width_in_pixels != last_w as u16 && last_w != 0 {
                            last_w = scr.width_in_pixels as u32;
                            let _ = tx.send(DisplayChangeEvent::ResolutionChanged);
                        }
                        last_w = scr.width_in_pixels as u32;
                    }
                }
            });
        }
        rx
    }

    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String> {
        let monitors = self.enumerate_monitors();
        let mon = monitors
            .iter()
            .find(|m| m.id == monitor_id)
            .cloned()
            .ok_or_else(|| "Monitor not found".to_string())?;

        let screen = &self.conn.setup().roots[self.screen_num];
        let visual = self.argb_visual.unwrap_or(screen.root_visual);
        let colormap = self.argb_colormap.unwrap_or(screen.default_colormap);

        let win = self
            .conn
            .generate_id()
            .map_err(|e| format!("generate_id: {}", e))?;

        let aux = CreateWindowAux::new()
            .background_pixel(0x00000000)
            .border_pixel(0)
            .colormap(colormap)
            .override_redirect(xproto::Bool32::from(true));

        self.conn
            .create_window(
                32,
                win,
                self.root,
                mon.x as i16,
                mon.y as i16,
                mon.width.max(1) as u16,
                mon.height.max(1) as u16,
                0,
                xproto::WindowClass::INPUT_OUTPUT,
                visual,
                &aux,
            )
            .map_err(|e| format!("create_window: {}", e))?;

        // Set _NET_WM_WINDOW_TYPE_DOCK
        let type_atom = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom);
        let dock_atom = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DOCK")
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom);
        if let (Some(ta), Some(da)) = (type_atom, dock_atom) {
            let _ = self.conn.change_property32(
                PropMode::REPLACE,
                win,
                ta,
                xproto::AtomEnum::ATOM,
                &[da],
            );
        }

        // Make click-through via Shape (empty input region)
        if self
            .conn
            .extension_information(shape::X11_EXTENSION_NAME)
            .is_ok_and(|e| e.is_some())
        {
            let _ = self.conn.shape_mask(
                shape::SO::SET,
                shape::SK::INPUT,
                win,
                0,
                0,
                0u32, // none pixmap
            );
        }

        let _ = self.conn.map_window(win);
        let _ = self.conn.flush();

        Ok(OverlayHandle(win as u64))
    }

    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32) {
        let win = handle.0 as u32;
        let gc = self.ensure_overlay_gc(win);
        if gc == 0 {
            return;
        }

        // Convert RGBA → BGRA for X11
        let mut buf = Vec::with_capacity(pixels.len());
        for chunk in pixels.chunks_exact(4) {
            buf.push(chunk[2]);
            buf.push(chunk[1]);
            buf.push(chunk[0]);
            buf.push(chunk[3]);
        }

        let _ = self.conn.put_image(
            xproto::ImageFormat::Z_PIXMAP,
            win,
            gc,
            w as u16,
            h as u16,
            0,
            0,
            0, // left_pad
            32,
            &buf,
        );
        let _ = self.conn.flush();
    }

    fn destroy_overlay_window(&self, handle: OverlayHandle) {
        let _ = self.conn.destroy_window(handle.0 as u32);
        let _ = self.conn.flush();
    }

    fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        let autostart_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("autostart");
        let desktop_path = autostart_dir.join("grid-screen.desktop");

        if enabled {
            let exe = std::env::current_exe().map_err(|e| format!("{}", e))?;
            let content = format!(
                "[Desktop Entry]\nType=Application\nName=Grid Screen\nComment=Cross-platform window zone manager\nExec={}\nTerminal=false\nCategories=Utility;\nX-GNOME-Autostart-enabled=true\n",
                exe.display()
            );
            std::fs::create_dir_all(&autostart_dir).map_err(|e| format!("{}", e))?;
            std::fs::write(&desktop_path, content).map_err(|e| format!("{}", e))?;
        } else {
            let _ = std::fs::remove_file(&desktop_path);
        }
        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────

fn find_argb_visual(conn: &RustConnection, screen_num: usize) -> (Option<Visualid>, Option<u32>) {
    let screen = &conn.setup().roots[screen_num];
    for depth in &screen.allowed_depths {
        if depth.depth == 32 {
            for vis in &depth.visuals {
                if vis.class == VisualClass::TRUE_COLOR {
                    let mask = vis.red_mask | vis.green_mask | vis.blue_mask;
                    if mask.count_ones() == 24 {
                        return (Some(vis.visual_id), Some(screen.default_colormap));
                    }
                }
            }
        }
    }
    (None, None)
}

fn get_window_title(conn: &RustConnection, window: u32) -> String {
    let utf8_atom = conn
        .intern_atom(false, b"_NET_WM_NAME")
        .ok()
        .and_then(|c| c.reply().ok());

    if let Some(ref atom) = utf8_atom {
        let reply = conn
            .get_property(false, window, atom.atom, xproto::AtomEnum::STRING, 0, 256)
            .ok()
            .and_then(|c| c.reply().ok());
        if let Some(prop) = reply {
            if prop.length > 0 {
                return String::from_utf8_lossy(&prop.value).to_string();
            }
        }
    }

    let reply = conn
        .get_property(
            false,
            window,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            0,
            256,
        )
        .ok()
        .and_then(|c| c.reply().ok());

    if let Some(prop) = reply {
        if prop.length > 0 {
            return String::from_utf8_lossy(&prop.value).to_string();
        }
    }

    String::new()
}
