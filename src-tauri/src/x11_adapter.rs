use shared_types::*;
use crate::platform_adapter::{PlatformAdapter, WorkspaceId, EventStream};
use tokio::sync::broadcast;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::connection::Connection;
use x11rb::atom_manager;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        _NET_CLIENT_LIST,
        _NET_CLIENT_LIST_STACKING,
        _NET_WM_NAME,
        _NET_WM_VISIBLE_NAME,
        _NET_WM_STATE,
        _NET_WM_STATE_HIDDEN,
        _NET_WM_STATE_MAXIMIZED_VERT,
        _NET_WM_STATE_MAXIMIZED_HORZ,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_NORMAL,
        _NET_WM_WINDOW_TYPE_DIALOG,
        _NET_WM_WINDOW_TYPE_DESKTOP,
        _NET_WM_WINDOW_TYPE_DOCK,
        _NET_WM_WINDOW_TYPE_MENU,
        _NET_WM_WINDOW_TYPE_TOOLBAR,
        _NET_WM_WINDOW_TYPE_NOTIFICATION,
        _NET_WM_WINDOW_TYPE_UTILITY,
        _NET_WM_WINDOW_TYPE_SPLASH,
        _NET_WM_ALLOWED_ACTIONS,
        _NET_WM_ACTION_MOVE,
        _NET_WM_ACTION_RESIZE,
        _NET_WM_ACTION_MINIMIZE,
        _NET_WM_ACTION_FULLSCREEN,
        _NET_FRAME_EXTENTS,
        _NET_WORKAREA,
        _NET_CURRENT_DESKTOP,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_DESKTOP_NAMES,
        _NET_SUPPORTING_WM_CHECK,
        _NET_WM_PID,
        WM_NAME,
        WM_STATE,
        WM_CLASS,
        UTF8_STRING,
    }
}

pub struct X11Adapter {
    conn: RustConnection,
    atoms: AtomCollection,
    screen_num: usize,
    root: u32,
    system_status: SystemStatus,
    #[allow(dead_code)]
    workspace_tx: broadcast::Sender<WorkspaceChangedPayload>,
    #[allow(dead_code)]
    screen_tx: broadcast::Sender<ScreenChangedPayload>,
}

impl X11Adapter {
    pub fn new() -> Result<Self, String> {
        let display = std::env::var("DISPLAY").unwrap_or_default();
        if display.is_empty() {
            return Err("No X11 display available — $DISPLAY is not set".into());
        }

        let (conn, screen_num) = x11rb::connect(None)
            .map_err(|e| format!("Failed to connect to X11 server (DISPLAY={}): {}", display, e))?;

        let atoms = AtomCollection::new(&conn)
            .map_err(|e| format!("Failed to intern X11 atoms: {}", e))?
            .reply()
            .map_err(|e| format!("Failed to intern X11 atoms: {}", e))?;

        let root = conn.setup().roots[screen_num].root;
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "tty".into());

        let wm_name = Self::get_wm_name(&conn, &atoms).unwrap_or_else(|| "unknown".into());
        let ewmh_support = if Self::check_ewmh(&conn, &atoms, root) {
            "Full"
        } else {
            "Partial"
        };

        let (workspace_tx, _) = broadcast::channel(16);
        let (screen_tx, _) = broadcast::channel(16);

        let screens = Self::detect_screens(&conn, screen_num, &atoms, root);
        let connected_screens = screens.iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let workspace = Self::get_current_workspace(&conn, &atoms, root)
            .unwrap_or_else(|| "1".into());

        let xrandr_available = false; // x11rb 0.13: randr extension detection deferred

        Ok(X11Adapter {
            conn,
            atoms,
            screen_num,
            root,
            system_status: SystemStatus {
                session_type,
                ewmh_support: ewmh_support.into(),
                wm_name,
                xrandr_available,
                workspace: workspace.clone(),
                connected_screens,
                errors: vec![],
            },
            workspace_tx,
            screen_tx,
        })
    }

    fn get_wm_name(conn: &RustConnection, atoms: &AtomCollection) -> Option<String> {
        let reply = conn.get_property(
            false,
            conn.setup().roots[0].root,
            atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            0,
            1,
        ).ok()?.reply().ok()?;

        if reply.value32().is_none() {
            return None;
        }
        let wm_check = reply.value32()?.next()?;

        let name = conn.get_property(
            false,
            wm_check,
            atoms._NET_WM_NAME,
            atoms.UTF8_STRING,
            0,
            256,
        ).ok()?.reply().ok()?;

        String::from_utf8(name.value.to_vec()).ok()
    }

    fn check_ewmh(conn: &RustConnection, atoms: &AtomCollection, root: u32) -> bool {
        conn.get_property(
            false,
            root,
            atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            0,
            1,
        ).ok().and_then(|r| r.reply().ok()).is_some()
    }

    fn detect_screens(conn: &RustConnection, screen_num: usize, _atoms: &AtomCollection, root: u32) -> Vec<ScreenInfo> {
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];

        let work_area = Self::get_work_area(conn, root, screen.width_in_pixels as i32, screen.height_in_pixels as i32);

        vec![ScreenInfo {
            id: format!(":0.{}", screen_num),
            label: format!("Screen {}", screen_num),
            resolution: format!("{} x {}", screen.width_in_pixels, screen.height_in_pixels),
            work_area,
        }]
    }

    fn get_work_area(conn: &RustConnection, root: u32, width: i32, height: i32) -> Rect {
        let atoms = AtomCollection::new(conn).ok().and_then(|ac| ac.reply().ok());
        if let Some(atoms) = atoms {
            if let Ok(reply) = conn.get_property(false, root, atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, 4) {
                if let Ok(reply) = reply.reply() {
                    let data: Vec<u32> = reply.value32().map(|i| i.collect()).unwrap_or_default();
                    if data.len() >= 4 {
                        return Rect {
                            x: data[0] as i32,
                            y: data[1] as i32,
                            width: data[2],
                            height: data[3],
                        };
                    }
                }
            }
        }
        Rect { x: 0, y: 0, width: width as u32, height: height as u32 }
    }

    fn get_current_workspace(conn: &RustConnection, atoms: &AtomCollection, root: u32) -> Option<String> {
        let reply = conn.get_property(
            false, root,
            atoms._NET_CURRENT_DESKTOP,
            AtomEnum::CARDINAL, 0, 1,
        ).ok()?.reply().ok()?;

        let desktop = reply.value32()?.next()?;
        Some((desktop + 1).to_string())
    }

    fn get_window_title(conn: &RustConnection, atoms: &AtomCollection, window: u32) -> String {
        // Try _NET_WM_NAME first
        if let Ok(reply) = conn.get_property(false, window, atoms._NET_WM_NAME, atoms.UTF8_STRING, 0, 256) {
            if let Ok(reply) = reply.reply() {
                if let Ok(s) = String::from_utf8(reply.value.to_vec()) {
                    if !s.is_empty() { return s; }
                }
            }
        }
        // Fallback to WM_NAME
        if let Ok(reply) = conn.get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 256) {
            if let Ok(reply) = reply.reply() {
                if let Ok(s) = String::from_utf8(reply.value.to_vec()) {
                    if !s.is_empty() { return s; }
                }
            }
        }
        String::new()
    }

    fn get_app_name(conn: &RustConnection, _atoms: &AtomCollection, window: u32) -> String {
        // Try WM_CLASS
        if let Ok(reply) = conn.get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256) {
            if let Ok(reply) = reply.reply() {
                let value = String::from_utf8_lossy(&reply.value);
                // WM_CLASS is "instance\0class"
                if let Some(name) = value.split('\0').nth(1) {
                    if !name.is_empty() { return name.to_string(); }
                }
                if let Some(name) = value.split('\0').next() {
                    if !name.is_empty() { return name.to_string(); }
                }
            }
        }
        "Unknown".to_string()
    }

    #[allow(dead_code)]
    fn get_pid(conn: &RustConnection, atoms: &AtomCollection, window: u32) -> Option<u32> {
        conn.get_property(false, window, atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, 1)
            .ok()?
            .reply().ok()?
            .value32()?
            .next()
    }

    fn get_icon_color(app_name: &str) -> String {
        let mut hash: u32 = 0;
        for b in app_name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u32);
        }
        let hue = hash % 360;
        format!("hsl({}, 55%, 50%)", hue)
    }

    fn is_eligible(&self, _window: u32, wm_state_set: &std::collections::HashSet<u32>, window_type_atom: Option<u32>) -> bool {
        // Skip fullscreen windows
        if wm_state_set.contains(&self.atoms._NET_WM_STATE_FULLSCREEN) {
            return false;
        }

        // Skip non-normal window types
        if let Some(wtype) = window_type_atom {
            let skip_types = [
                self.atoms._NET_WM_WINDOW_TYPE_DESKTOP,
                self.atoms._NET_WM_WINDOW_TYPE_DOCK,
                self.atoms._NET_WM_WINDOW_TYPE_MENU,
                self.atoms._NET_WM_WINDOW_TYPE_TOOLBAR,
                self.atoms._NET_WM_WINDOW_TYPE_NOTIFICATION,
                self.atoms._NET_WM_WINDOW_TYPE_UTILITY,
                self.atoms._NET_WM_WINDOW_TYPE_SPLASH,
            ];
            if skip_types.contains(&wtype) {
                return false;
            }
        }

        true
    }
}

impl PlatformAdapter for X11Adapter {
    fn enumerate_screens(&self) -> Vec<ScreenInfo> {
        Self::detect_screens(&self.conn, self.screen_num, &self.atoms, self.root)
    }

    fn current_workspace(&self) -> WorkspaceId {
        Self::get_current_workspace(&self.conn, &self.atoms, self.root)
            .unwrap_or_else(|| "1".into())
    }

    fn enumerate_windows(&self, _workspace: &str) -> Vec<WindowDescriptor> {
        let mut windows = Vec::new();

        let window_ids = self.get_client_list();

        for &win_id in &window_ids {
            if let Some(desc) = self.build_window_descriptor(win_id) {
                windows.push(desc);
            }
        }

        windows
    }

    fn get_window_state(&self, window_id: &str) -> Option<WindowState> {
        let win: u32 = window_id.parse().ok()?;
        let _attr = self.conn.get_window_attributes(win).ok()?.reply().ok()?;

        let (minimized, maximized, fullscreen) = self.get_wm_state(win);
        let (movable, resizable) = self.get_allowed_actions(win);

        Some(WindowState {
            minimized,
            maximized,
            fullscreen,
            movable,
            resizable,
        })
    }

    fn get_frame_extents(&self, window_id: &str) -> Rect {
        let win: u32 = match window_id.parse() {
            Ok(w) => w,
            Err(_) => return Rect { x: 0, y: 0, width: 0, height: 0 },
        };

        let reply = match self.conn.get_property(
            false, win,
            self.atoms._NET_FRAME_EXTENTS,
            AtomEnum::CARDINAL, 0, 4,
        ) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(_) => return Rect { x: 0, y: 0, width: 0, height: 0 },
            },
            Err(_) => return Rect { x: 0, y: 0, width: 0, height: 0 },
        };

        let data: Vec<u32> = reply.value32().map(|i| i.collect()).unwrap_or_default();
        if data.len() >= 4 {
            Rect {
                x: data[0] as i32,  // left
                y: data[2] as i32,  // top
                width: data[1],     // right
                height: data[3],    // bottom
            }
        } else {
            Rect { x: 0, y: 0, width: 0, height: 0 }
        }
    }

    fn restore_window(&self, window_id: &str) {
        let win: u32 = match window_id.parse() {
            Ok(w) => w,
            Err(_) => return,
        };
        // Send _NET_ACTIVE_WINDOW to restore minimized windows
        let _ = self.conn.map_window(win);
    }

    fn move_resize_window(&self, window_id: &str, rect: Rect) -> Result<Rect, String> {
        let win: u32 = window_id.parse()
            .map_err(|_| format!("Invalid window ID: {}", window_id))?;

        // Use _NET_MOVERESIZE_WINDOW client message
        let data = [
            0u32, // source indication
            rect.x as u32,
            rect.y as u32,
            rect.width,
            rect.height,
        ];

        let event = ClientMessageEvent::new(
            32,
            win,
            self.atoms._NET_WM_STATE,
            ClientMessageData::from(data),
        );

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            &event,
        ).map_err(|e| format!("X11 send_event failed: {}", e))?;

        self.conn.flush().map_err(|e| format!("X11 flush failed: {}", e))?;

        Ok(rect)
    }

    fn subscribe_workspace_events(&self) -> EventStream<WorkspaceChangedPayload> {
        self.workspace_tx.subscribe()
    }

    fn subscribe_screen_events(&self) -> EventStream<ScreenChangedPayload> {
        self.screen_tx.subscribe()
    }

    fn detect_capabilities(&self) -> SystemStatus {
        self.system_status.clone()
    }
}

impl X11Adapter {
    fn get_client_list(&self) -> Vec<u32> {
        if let Ok(reply) = self.conn.get_property(
            false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, 1024,
        ) {
            if let Ok(reply) = reply.reply() {
                let ids: Vec<u32> = reply.value32().map(|i| i.collect()).unwrap_or_default();
                if !ids.is_empty() { return ids; }
            }
        }
        self.query_tree_windows()
    }

    fn query_tree_windows(&self) -> Vec<u32> {
        let reply = match self.conn.query_tree(self.root) {
            Ok(r) => match r.reply() { Ok(r) => r, Err(_) => return Vec::new() },
            Err(_) => return Vec::new(),
        };
        let mut ids = Vec::new();
        for &child in &reply.children {
            if let Ok(attrs) = self.conn.get_window_attributes(child) {
                if let Ok(attrs) = attrs.reply() {
                    if attrs.map_state == MapState::VIEWABLE
                        && !attrs.override_redirect
                    { ids.push(child); }
                }
            }
        }
        ids
    }

    fn build_window_descriptor(&self, win_id: u32) -> Option<WindowDescriptor> {
        let title = Self::get_window_title(&self.conn, &self.atoms, win_id);
        let app_name = Self::get_app_name(&self.conn, &self.atoms, win_id);
        let icon_color = Self::get_icon_color(&app_name);
        let (minimized, maximized, fullscreen) = self.get_wm_state(win_id);
        let (movable, resizable) = self.get_allowed_actions(win_id);
        if !self.is_eligible(win_id, &self.get_wm_state_set(win_id), self.get_window_type_atom(win_id)) {
            return None;
        }
        Some(WindowDescriptor {
            id: win_id.to_string(), app_name, title, icon_color,
            state: WindowState { minimized, maximized, fullscreen, movable, resizable },
        })
    }

    fn get_wm_state(&self, window: u32) -> (bool, bool, bool) {
        let reply = match self.conn.get_property(
            false, window,
            self.atoms._NET_WM_STATE,
            AtomEnum::ATOM, 0, 32,
        ) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(_) => return (false, false, false),
            },
            Err(_) => return (false, false, false),
        };

        let atoms: Vec<u32> = reply.value32().map(|i| i.collect()).unwrap_or_default();
        let set: std::collections::HashSet<u32> = atoms.into_iter().collect();

        (
            set.contains(&self.atoms._NET_WM_STATE_HIDDEN),
            set.contains(&self.atoms._NET_WM_STATE_MAXIMIZED_VERT)
                && set.contains(&self.atoms._NET_WM_STATE_MAXIMIZED_HORZ),
            set.contains(&self.atoms._NET_WM_STATE_FULLSCREEN),
        )
    }

    fn get_wm_state_set(&self, window: u32) -> std::collections::HashSet<u32> {
        let reply = match self.conn.get_property(
            false, window,
            self.atoms._NET_WM_STATE,
            AtomEnum::ATOM, 0, 32,
        ) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(_) => return std::collections::HashSet::new(),
            },
            Err(_) => return std::collections::HashSet::new(),
        };
        reply.value32().map(|i| i.collect()).unwrap_or_default()
    }

    fn get_allowed_actions(&self, window: u32) -> (bool, bool) {
        let reply = match self.conn.get_property(
            false, window,
            self.atoms._NET_WM_ALLOWED_ACTIONS,
            AtomEnum::ATOM, 0, 16,
        ) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(_) => return (true, true), // assume allowed if can't query
            },
            Err(_) => return (true, true),
        };

        let atoms: Vec<u32> = reply.value32().map(|i| i.collect()).unwrap_or_default();
        (
            atoms.contains(&self.atoms._NET_WM_ACTION_MOVE),
            atoms.contains(&self.atoms._NET_WM_ACTION_RESIZE),
        )
    }

    fn get_window_type_atom(&self, window: u32) -> Option<u32> {
        let cookie = self.conn.get_property(
            false, window,
            self.atoms._NET_WM_WINDOW_TYPE,
            AtomEnum::ATOM, 0, 4,
        ).ok()?;
        let reply = cookie.reply().ok()?;
        let mut iter = reply.value32()?;
        iter.next()
    }
}
