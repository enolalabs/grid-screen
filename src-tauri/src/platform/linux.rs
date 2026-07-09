use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use tracing;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as RandrExt};
use x11rb::protocol::shape::{ConnectionExt as ShapeExt, SK};
use x11rb::protocol::xinerama::{self, ConnectionExt as XineramaExt};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use super::PlatformApi;
use crate::types::*;

struct X11Connection {
    conn: RustConnection,
    screen_num: usize,
    root: u32,
    atoms: X11Atoms,
    randr_resources: Arc<Mutex<Option<Vec<randr::MonitorInfo>>>>,
    move_event_source: mpsc::Sender<WindowMoveEvent>,
    display_change_source: mpsc::Sender<DisplayChangeEvent>,
}

#[derive(Clone)]
struct X11Atoms {
    net_wm_name: u32,
    net_wm_window_type: u32,
    net_wm_window_type_dock: u32,
    utf8_string: u32,
}

impl X11Connection {
    fn connect() -> Result<Self, String> {
        let (conn, screen_num) = RustConnection::connect(None)
            .map_err(|e| format!("Failed to connect to X server: {}", e))?;

        let root = conn.setup().roots[screen_num].root;

        let atoms = X11Atoms {
            net_wm_name: intern_atom(&conn, "_NET_WM_NAME"),
            net_wm_window_type: intern_atom(&conn, "_NET_WM_WINDOW_TYPE"),
            net_wm_window_type_dock: intern_atom(&conn, "_NET_WM_WINDOW_TYPE_DOCK"),
            utf8_string: intern_atom(&conn, "UTF8_STRING"),
        };

        let (move_tx, _) = mpsc::channel();
        let (display_tx, _) = mpsc::channel();

        Ok(Self {
            conn,
            screen_num,
            root,
            atoms,
            randr_resources: Arc::new(Mutex::new(None)),
            move_event_source: move_tx,
            display_change_source: display_tx,
        })
    }
}

fn intern_atom(conn: &RustConnection, name: &str) -> u32 {
    conn.intern_atom(false, name.as_bytes())
        .map(|r| r.reply().map(|r| r.atom).unwrap_or(0))
        .unwrap_or(0)
}

pub struct LinuxPlatformApi {
    x11: X11Connection,
}

impl LinuxPlatformApi {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            x11: X11Connection::connect()?,
        })
    }

    fn get_window_title(&self, window: u32) -> String {
        let atoms = &self.x11.atoms;
        let reply = self.x11.conn.get_property(
            false, window, atoms.net_wm_name, atoms.utf8_string, 0, 256,
        ).ok().and_then(|r| r.reply().ok());

        reply.map(|r| String::from_utf8_lossy(&r.value).to_string()).unwrap_or_default()
    }

    fn translate_coords(&self, src: u32, dst: u32, x: i16, y: i16) -> (i16, i16) {
        self.x11.conn.translate_coordinates(src, dst, x, y)
            .ok()
            .and_then(|r| r.reply().ok())
            .map(|r| (r.dst_x, r.dst_y))
            .unwrap_or((x, y))
    }
}

impl PlatformApi for LinuxPlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor> {
        // Try RandR first, fall back to Xinerama
        if let Ok(monitors) = self.x11.conn.randr_get_monitors(self.x11.root, true) {
            if let Ok(reply) = monitors.reply() {
                let randr_resources = &self.x11.randr_resources;
                if let Ok(mut res) = randr_resources.lock() {
                    *res = Some(reply.monitors.clone());
                }

                return reply.monitors.iter().map(|m| {
                    let dpi_x = if m.width_in_millimeters > 0 {
                        m.width as f64 / (m.width_in_millimeters as f64 / 25.4)
                    } else { 96.0 };
                    let dpi_scale = (dpi_x / 96.0).max(1.0);
                    Monitor {
                        id: MonitorId(uuid::Uuid::new_v4()),
                        name: String::from_utf8_lossy(&m.name).to_string(),
                        x: m.x as i32,
                        y: m.y as i32,
                        width: m.width as u32,
                        height: m.height as u32,
                        dpi_scale,
                        is_primary: m.primary == 1,
                    }
                }).collect();
            }
        }

        // Fallback to Xinerama
        if let Ok(screens) = self.x11.conn.xinerama_query_screens() {
            if let Ok(reply) = screens.reply() {
                return reply.screen_info.iter().enumerate().map(|(i, s)| {
                    Monitor {
                        id: MonitorId(uuid::Uuid::new_v4()),
                        name: format!("Screen {}", i + 1),
                        x: s.x_org as i32,
                        y: s.y_org as i32,
                        width: s.width as u32,
                        height: s.height as u32,
                        dpi_scale: 1.0,
                        is_primary: i == 0,
                    }
                }).collect();
            }
        }

        // Ultimate fallback: single screen from root window geometry
        if let Ok(geo) = self.x11.conn.get_geometry(self.x11.root) {
            if let Ok(reply) = geo.reply() {
                let screen = &self.x11.conn.setup().roots[self.x11.screen_num];
                return vec![Monitor {
                    id: MonitorId(uuid::Uuid::new_v4()),
                    name: "Screen 1".into(),
                    x: 0,
                    y: 0,
                    width: screen.width_in_pixels as u32,
                    height: screen.height_in_pixels as u32,
                    dpi_scale: 1.0,
                    is_primary: true,
                }];
            }
            return vec![];
        }

        vec![]
    }

    fn enumerate_windows(&self) -> Vec<Window> {
        let root = self.x11.root;
        let mut windows = Vec::new();

        if let Ok(tree) = self.x11.conn.query_tree(root) {
            if let Ok(reply) = tree.reply() {
                for child in reply.children {
                    // Filter out windows with no size or override-redirect
                    if let Ok(attr) = self.x11.conn.get_window_attributes(child) {
                        if let Ok(attr_reply) = attr.reply() {
                            if attr_reply.map_state == MapState::UNMAPPED {
                                continue;
                            }
                        }
                    }

                    if let Ok(geo) = self.x11.conn.get_geometry(child) {
                        if let Ok(geo_reply) = geo.reply() {
                            if geo_reply.width == 0 || geo_reply.height == 0 {
                                continue;
                            }

                            let (abs_x, abs_y) = self.translate_coords(child, root, 0, 0);

                            let title = self.get_window_title(child.resource_id());
                            windows.push(Window {
                                handle: WindowHandle(child.resource_id() as u64),
                                title,
                                rect: Rect {
                                    x: abs_x as i32,
                                    y: abs_y as i32,
                                    width: geo_reply.width as u32,
                                    height: geo_reply.height as u32,
                                },
                                is_visible: true,
                            });
                        }
                    }
                }
            }
        }

        windows
    }

    fn move_window(&self, handle: WindowHandle, rect: Rect) {
        let window = handle.0 as u32;
        let values = ConfigureWindowAux::default()
            .x(rect.x)
            .y(rect.y)
            .width(rect.width)
            .height(rect.height);
        let _ = self.x11.conn.configure_window(window, &values);
        self.x11.conn.flush().ok();
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        self.x11.conn.query_pointer(self.x11.root)
            .ok()
            .and_then(|r| r.reply().ok())
            .map(|r| (r.root_x as i32, r.root_y as i32))
            .unwrap_or((0, 0))
    }

    fn is_mouse_button_down(&self) -> bool {
        self.x11.conn.query_pointer(self.x11.root)
            .ok()
            .and_then(|r| r.reply().ok())
            .map(|r| (r.mask & 0x1F00) != 0) // Any button mask
            .unwrap_or(false)
    }

    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
        let (tx, rx) = mpsc::channel();
        let screen_num = self.x11.screen_num;

        thread::spawn(move || {
            let (conn, _) = match RustConnection::connect(None) {
                Ok(c) => c,
                Err(_) => return,
            };

            let root = conn.setup().roots[screen_num].root;
            let change = ChangeWindowAttributesAux::default()
                .event_mask(EventMask::SUBSTRUCTURE_NOTIFY);
            conn.change_window_attributes(root, &change).ok();
            conn.flush().ok();

            let mut drag_window: Option<u32> = None;

            loop {
                let event = match conn.wait_for_event() {
                    Ok(e) => e,
                    Err(_) => break,
                };

                if let Event::ConfigureNotify(ev) = event {
                    let window = ev.window;

                    let btn_down = conn.query_pointer(root).ok()
                        .and_then(|r| r.reply().ok())
                        .map(|r| (r.mask & 0x1F00) != 0)
                        .unwrap_or(false);

                    let rect = Rect {
                        x: ev.x as i32,
                        y: ev.y as i32,
                        width: ev.width as u32,
                        height: ev.height as u32,
                    };

                    if btn_down {
                        if drag_window != Some(window) {
                            drag_window = Some(window);
                            let _ = tx.send(WindowMoveEvent::DragStart {
                                handle: WindowHandle(window as u64),
                                rect,
                            });
                        } else {
                            let _ = tx.send(WindowMoveEvent::DragMove {
                                handle: WindowHandle(window as u64),
                                rect,
                            });
                        }
                    } else if let Some(dw) = drag_window {
                        if dw == window {
                            let _ = tx.send(WindowMoveEvent::DragEnd {
                                handle: WindowHandle(window as u64),
                                rect,
                            });
                        }
                        drag_window = None;
                    }
                }
            }
        });

        rx
    }

    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
        let (tx, rx) = mpsc::channel();
        let screen_num = self.x11.screen_num;

        thread::spawn(move || {
            let conn = match RustConnection::connect(None) {
                Ok((c, _)) => c,
                Err(_) => return,
            };

            let root = conn.setup().roots[screen_num].root;
            conn.randr_select_input(root, randr::NotifyMask::SCREEN_CHANGE).ok();
            conn.flush().ok();

            loop {
                if conn.wait_for_event().is_err() {
                    break;
                }
                let _ = tx.send(DisplayChangeEvent::Connected);
            }
        });

        rx
    }

    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String> {
        let conn = &self.x11.conn;
        let root = self.x11.root;
        let screen = &conn.setup().roots[self.x11.screen_num];

        let window_id = conn.generate_id()
            .map_err(|e| format!("Failed to generate X11 window id: {}", e))?;

        // Find the monitor for positioning
        let mons = self.enumerate_monitors();
        let mon = mons.iter().find(|m| m.id == monitor_id)
            .cloned();

        let (mon_x, mon_y, mon_w, mon_h) = mon.map_or(
            (0i32, 0i32, screen.width_in_pixels as u32, screen.height_in_pixels as u32),
            |m| (m.x, m.y, m.width, m.height),
        );

        let values = CreateWindowAux::default()
            .override_redirect(1u32)
            .background_pixel(screen.black_pixel)
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::POINTER_MOTION,
            );

        conn.create_window(
            CopyFromParent::COPY_DEPTH_FROM_PARENT as u8,
            window_id,
            root,
            mon_x as i16,
            mon_y as i16,
            mon_w as u16,
            mon_h as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            CopyFromParent::COPY_FROM_PARENT,
            &values,
        )
        .map_err(|e| format!("Failed to create overlay window: {}", e))?;

        // Make window click-through: set empty input shape
        conn.shape_rectangles(
            SK::BOUNDING,
            ShapeOperation::SET,
            ShapeKind::INPUT as u8,
            window_id,
            0,
            0,
            0,
            0,
            &[], // Empty rectangles = no input → click-through
        )
        .ok();

        // Keep window below normal windows
        let below = StackMode::BELOW;
        conn.configure_window(
            window_id,
            &ConfigureWindowAux::default()
                .stack_mode(below),
        ).ok();

        // Map the window
        conn.map_window(window_id).ok();
        conn.flush().ok();

        Ok(OverlayHandle(window_id.resource_id() as u64))
    }

    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32) {
        let window = handle.0 as u32;
        let conn = &self.x11.conn;

        // Create a pixmap and draw into it with PutImage
        // tiny-skia outputs RGBA bytes; XPutImage expects the server's native format
        let gc = conn.generate_id().ok();
        if let Some(gc_id) = gc {
            conn.create_gc(
                gc_id,
                self.x11.root,
                &CreateGCAux::default(),
            ).ok();

            // PutImage: send pixel data to window
            conn.put_image(
                ImageFormat::Z_PIXMAP,
                window,
                gc_id,
                w as u16,
                h as u16,
                0,
                0,
                0,
                24, // depth
                pixels,
            ).ok();

            conn.flush().ok();
        }
    }

    fn destroy_overlay_window(&self, handle: OverlayHandle) {
        let window = handle.0 as u32;
        self.x11.conn.destroy_window(window).ok();
        self.x11.conn.flush().ok();
    }

    fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        let autostart_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("autostart");

        let desktop_file = autostart_dir.join("grid-screen.desktop");

        if enabled {
            std::fs::create_dir_all(&autostart_dir)
                .map_err(|e| format!("Cannot create autostart dir: {}", e))?;

            let exe = std::env::current_exe()
                .map_err(|e| format!("Cannot determine exe path: {}", e))?;

            let content = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Name=Grid Screen\n\
                 Comment=Window zone management\n\
                 Exec={}\n\
                 Terminal=false\n\
                 X-GNOME-Autostart-enabled=true\n",
                exe.display()
            );

            std::fs::write(&desktop_file, content)
                .map_err(|e| format!("Cannot write autostart file: {}", e))?;
        } else {
            if desktop_file.exists() {
                std::fs::remove_file(&desktop_file).ok();
            }
        }

        Ok(())
    }
}
