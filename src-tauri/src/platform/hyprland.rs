use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as XProtoExt;
use x11rb::protocol::xproto::KeyButMask;

use super::PlatformApi;
use crate::types::*;

/// Check if we are running under Hyprland
pub fn is_hyprland_session() -> bool {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
}

/// Path to the Hyprland IPC socket for events
fn hypr_socket_path() -> Option<PathBuf> {
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(PathBuf::from(runtime).join("hypr").join(sig).join(".socket2.sock"))
}

/// Run `hyprctl` and return stdout
fn hyprctl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|e| format!("hyprctl failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hyprctl error: {}", stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── JSON response types ─────────────────────────────────────

#[derive(Deserialize)]
struct HyprMonitor {
    name: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    #[serde(alias = "refreshRate")]
    refresh_rate: f64,
    scale: Option<f64>,
}

#[derive(Deserialize)]
struct HyprWindow {
    address: String,
    mapped: bool,
    hidden: bool,
    at: [i32; 2],
    size: [u32; 2],
    workspace: HyprWorkspace,
    class: String,
    title: String,
    floating: bool,
    focus_history_id: Option<u32>,
}

#[derive(Deserialize)]
struct HyprWorkspace {
    id: i32,
    name: String,
}

// ── Platform API ────────────────────────────────────────────

pub struct HyprlandPlatformApi {
    is_mouse_down_fallback: Arc<std::sync::atomic::AtomicBool>,
}

impl HyprlandPlatformApi {
    pub fn new() -> Result<Self, String> {
        if !is_hyprland_session() {
            return Err("Not a Hyprland session".into());
        }
        // Verify hyprctl works
        hyprctl(&["monitors"]).map_err(|e| format!("Hyprland check: {}", e))?;
        Ok(Self {
            is_mouse_down_fallback: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Detect mouse button state.
    /// On Hyprland we try X11 (XWayland) QueryPointer first,
    /// fall back to tracked state from IPC events.
    fn check_mouse_button(fallback: &std::sync::atomic::AtomicBool) -> bool {
        // Try X11 QueryPointer via XWayland
        if std::env::var("DISPLAY").ok().map_or(false, |d| !d.is_empty()) {
            if let Ok((conn, _)) = x11rb::connect(None) {
                let screen = &conn.setup().roots[0];
                if let Some(reply) = conn.query_pointer(screen.root).ok().and_then(|c| c.reply().ok()) {
                    return reply.mask.contains(KeyButMask::BUTTON1);
                }
            }
        }
        fallback.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl PlatformApi for HyprlandPlatformApi {
    fn enumerate_monitors(&self) -> Vec<Monitor> {
        let output = match hyprctl(&["monitors", "all", "-j"]) {
            Ok(o) => o,
            Err(_) => return vec![],
        };
        let monitors: Vec<HyprMonitor> = match serde_json::from_str(&output) {
            Ok(m) => m,
            Err(_) => return vec![],
        };
        let mut result = Vec::new();
        for (i, m) in monitors.iter().enumerate() {
            result.push(Monitor {
                id: MonitorId::from_name(&m.name),
                name: m.name.clone(),
                x: m.x,
                y: m.y,
                width: m.width,
                height: m.height,
                dpi_scale: m.scale.unwrap_or(1.0),
                is_primary: i == 0,
            });
        }
        result
    }

    fn enumerate_windows(&self) -> Vec<Window> {
        let output = match hyprctl(&["clients", "-j"]) {
            Ok(o) => o,
            Err(_) => return vec![],
        };
        let windows: Vec<HyprWindow> = match serde_json::from_str(&output) {
            Ok(w) => w,
            Err(_) => return vec![],
        };
        let mut result = Vec::new();
        for w in &windows {
            if !w.mapped || w.hidden {
                continue;
            }
            let title = if w.title.is_empty() { w.class.clone() } else { w.title.clone() };
            if title.is_empty() {
                continue;
            }
            // Use the address as a stable handle
            let handle_val = simple_hash(&w.address);
            result.push(Window {
                handle: WindowHandle(handle_val),
                title,
                rect: Rect {
                    x: w.at[0],
                    y: w.at[1],
                    width: w.size[0],
                    height: w.size[1],
                },
                is_visible: true,
            });
        }
        result
    }

    fn move_window(&self, handle: WindowHandle, rect: Rect) {
        let hex = format!("0x{:x}", handle.0);
        // Use hyprctl dispatch movetoworkspace or movewindowpixel
        // We need to move and resize the window precisely
        // hyprctl dispatch movewindowpixel and resizewindowpixel
        let _ = hyprctl(&[
            "dispatch",
            "movewindowpixel",
            &format!("{} {}", rect.x, rect.y),
            &format!("address:{}", hex),
        ]);
        let _ = hyprctl(&[
            "dispatch",
            "resizewindowpixel",
            &format!("{} {}", rect.width, rect.height),
            &format!("address:{}", hex),
        ]);
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        let output = match hyprctl(&["cursorpos"]) {
            Ok(o) => o,
            Err(_) => return (0, 0),
        };
        let trimmed = output.trim();
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() == 2 {
            let x = parts[0].trim().parse::<i32>().unwrap_or(0);
            let y = parts[1].trim().parse::<i32>().unwrap_or(0);
            return (x, y);
        }
        (0, 0)
    }

    fn is_mouse_button_down(&self) -> bool {
        Self::check_mouse_button(&self.is_mouse_down_fallback)
    }

    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
        let (tx, rx) = mpsc::channel();

        // Connect to Hyprland IPC event socket
        let socket_path = match hypr_socket_path() {
            Some(p) => p,
            None => {
                let (tx2, _) = mpsc::channel::<WindowMoveEvent>();
                std::mem::drop(tx2);
                return rx;
            }
        };

        let mouse_down_flag = Arc::clone(&self.is_mouse_down_fallback);

        thread::spawn(move || {
            let stream = match UnixStream::connect(&socket_path) {
                Ok(s) => s,
                Err(_) => return,
            };
            let reader = BufReader::new(stream);
            let mut known_windows: HashMap<u64, Rect> = HashMap::new();
            let mut drag_handle: Option<u64> = None;

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };

                // Parse Hyprland IPC event: event>>data
                let event_type = line.split(">>").next().unwrap_or("");

                // Track active window to know when drag starts
                if let Some(addr) = line.split(">>").nth(1) {
                    let handle_val = simple_hash(addr.split(',').next().unwrap_or(""));

                    match event_type {
                        "movewindow" | "changefloatingmode" => {
                            // Window moved or changed mode — poll position
                            let mouse_down = HyprlandPlatformApi::check_mouse_button(&mouse_down_flag);
                            let rect = get_window_rect(handle_val);

                            if let Some(r) = rect {
                                if mouse_down {
                                    if drag_handle == Some(handle_val) {
                                        let _ = tx.send(WindowMoveEvent::DragMove {
                                            handle: WindowHandle(handle_val),
                                            rect: r,
                                        });
                                    } else {
                                        drag_handle = Some(handle_val);
                                        let _ = tx.send(WindowMoveEvent::DragStart {
                                            handle: WindowHandle(handle_val),
                                            rect: r,
                                        });
                                    }
                                } else if let Some(dh) = drag_handle.take() {
                                    let final_rect = known_windows.get(&dh).copied().unwrap_or(r);
                                    let _ = tx.send(WindowMoveEvent::DragEnd {
                                        handle: WindowHandle(dh),
                                        rect: final_rect,
                                    });
                                }
                                known_windows.insert(handle_val, r);
                            }
                        }
                        "activewindow" => {
                            // Focus changed — may indicate end of drag
                            if let Some(dh) = drag_handle.take() {
                                let rect = known_windows.get(&dh).copied().unwrap_or(Rect {
                                    x: 0, y: 0, width: 0, height: 0,
                                });
                                let _ = tx.send(WindowMoveEvent::DragEnd {
                                    handle: WindowHandle(dh),
                                    rect,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        rx
    }

    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
        let (tx, rx) = mpsc::channel();

        let socket_path = match hypr_socket_path() {
            Some(p) => p,
            None => {
                let (tx2, _) = mpsc::channel::<DisplayChangeEvent>();
                std::mem::drop(tx2);
                return rx;
            }
        };

        thread::spawn(move || {
            loop {
                // Try to reconnect — handle socket reconnection
                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        let line = match line {
                            Ok(l) => l,
                            Err(_) => break,
                        };
                        let event_type = line.split(">>").next().unwrap_or("");
                        match event_type {
                            "monitoradded" => {
                                let _ = tx.send(DisplayChangeEvent::Connected);
                            }
                            "monitorremoved" => {
                                let _ = tx.send(DisplayChangeEvent::Disconnected);
                            }
                            "focusedmon" => {
                                // Could indicate resolution change too
                                let _ = tx.send(DisplayChangeEvent::ResolutionChanged);
                            }
                            _ => {}
                        }
                    }
                }
                thread::sleep(Duration::from_secs(5));
            }
        });

        rx
    }

    fn create_overlay_window(&self, _monitor_id: MonitorId) -> Result<OverlayHandle, String> {
        Err("Overlay not yet implemented on Wayland/Hyprland".into())
    }

    fn overlay_present(&self, _handle: &OverlayHandle, _pixels: &[u8], _w: u32, _h: u32) {}

    fn destroy_overlay_window(&self, _handle: OverlayHandle) {}

    fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        // Same desktop file approach as X11
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

// ── Helpers ─────────────────────────────────────────────────

fn simple_hash(s: &str) -> u64 {
    // Use a simple djb2 hash of the address string
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Poll a window's current rect via hyprctl clients -j
fn get_window_rect(handle_val: u64) -> Option<Rect> {
    let output = hyprctl(&["clients", "-j"]).ok()?;
    let windows: Vec<HyprWindow> = serde_json::from_str(&output).ok()?;
    let hex = format!("0x{:x}", handle_val);
    for w in &windows {
        if w.address == hex || simple_hash(&w.address) == handle_val {
            return Some(Rect {
                x: w.at[0],
                y: w.at[1],
                width: w.size[0],
                height: w.size[1],
            });
        }
    }
    None
}
