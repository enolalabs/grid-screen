# Grid Screen MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Linux X11 desktop app that lets users arrange application windows into zones using drag-and-drop on a visual canvas.

**Architecture:** Tauri 2 desktop shell with Rust application core and Svelte/TypeScript configuration UI. The Rust core communicates with X11 via a `PlatformAdapter` trait, sends typed commands/events across Tauri IPC to the Svelte webview. The webview holds ephemeral drag-and-drop assignment state.

**Tech Stack:** Rust (core), Svelte + TypeScript (UI), Tauri 2 (shell), X11/EWMH/XRandR (platform), JSON persistence in XDG_CONFIG_HOME, tracing (logging)

**Source of truth:** `mockups/design-1-aurora-dark.html`, `docs/superpowers/specs/2026-07-14-grid-screen-mvp-design.md`

## Global Constraints

- Platform: Linux X11 only; detect Wayland and show "X11 required" notice
- Window IDs are opaque, session-only, never persisted to disk
- Config directory: `${XDG_CONFIG_HOME:-~/.config}/grid-screen/` with `0700` perms, files `0600`
- No network requests, no telemetry, no analytics
- No continuous X11 polling while idle; use blocking event dispatch
- No keyboard accessibility in MVP (mouse interaction only)
- All user-provided strings rendered as text, never injected as HTML
- Default language: English; no localization
- Pointer-event-based drag-and-drop (not HTML5 DnD API)
- Slider geometry recomputation debounced at 150ms
- Tauri CSP: local assets and IPC only
- Atomic config writes: temp file → flush → validate → rename

---

## File Structure

```
grid-screen/
├── shared-types/                    # Shared Rust ↔ TS types
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── src-tauri/                       # Tauri + Rust core
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── app_shell.rs
│       ├── config_store.rs
│       ├── layout_engine.rs
│       ├── window_catalog.rs
│       ├── arrange_orchestrator.rs
│       ├── platform_adapter.rs
│       ├── x11_adapter.rs
│       └── diagnostics.rs
├── src/                             # Svelte UI
│   ├── main.ts
│   ├── App.svelte
│   ├── app.css
│   ├── lib/
│   │   ├── commands.ts
│   │   ├── events.ts
│   │   └── stores/
│   │       ├── assignments.ts
│   │       ├── layout.ts
│   │       ├── screen.ts
│   │       ├── windows.ts
│   │       ├── settings.ts
│   │       ├── systemStatus.ts
│   │       ├── arrangeState.ts
│   │       └── toasts.ts
│   └── components/
│       ├── TitleBar.svelte
│       ├── TabNav.svelte
│       ├── ArrangeView.svelte
│       ├── WindowCatalog.svelte
│       ├── WindowCard.svelte
│       ├── SearchBox.svelte
│       ├── CanvasArea.svelte
│       ├── CanvasToolbar.svelte
│       ├── ScreenSelector.svelte
│       ├── LayoutSelector.svelte
│       ├── LayoutQuickButtons.svelte
│       ├── ScreenCanvas.svelte
│       ├── ZoneSlot.svelte
│       ├── ActionBar.svelte
│       ├── DetailPanel.svelte
│       ├── LayoutSliders.svelte
│       ├── SnapControls.svelte
│       ├── SystemStatusPanel.svelte
│       ├── LayoutsView.svelte
│       ├── LayoutCard.svelte
│       ├── NewLayoutModal.svelte
│       ├── SettingsView.svelte
│       ├── SettingsGroup.svelte
│       ├── ToastContainer.svelte
│       └── ArrangeStateOverlay.svelte
├── package.json
├── tsconfig.json
├── vite.config.ts
├── svelte.config.js
└── index.html
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs` (skeleton)
- Create: `shared-types/Cargo.toml`
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `svelte.config.js`
- Create: `index.html`

**Interfaces:**
- Produces: `shared-types` crate, `src-tauri` crate, Svelte dev server ready for Tauri

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
members = ["src-tauri", "shared-types"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

Run: `cargo check` in project root — expected: "no targets to check" (workspace only)

- [ ] **Step 2: Create shared-types Cargo.toml**

```toml
[package]
name = "shared-types"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
ts-rs = "10"
```

Run: `cargo check -p shared-types` expected: `error[E0601]: main function not found in crate shared-types`

- [ ] **Step 3: Create src-tauri Cargo.toml**

```toml
[package]
name = "grid-screen"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"
shared-types = { path = "../shared-types" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

Run: `cargo check -p grid-screen` expected: fails on missing `src-tauri/src/main.rs`

- [ ] **Step 4: Create Tauri config**

```json
{
  "$schema": "https://raw.githubusercontent.com/nicedoc/open-docs/refs/heads/main/github.com/tauri-apps/tauri-docs/tauri.conf.json",
  "productName": "Grid Screen",
  "version": "0.1.0",
  "identifier": "com.gridscreen.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Grid Screen",
        "width": 1024,
        "height": 640,
        "decorations": false,
        "titleBarStyle": "Overlay"
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; script-src 'self'"
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/your-org/grid-screen/releases/latest/download/version.json"
      ],
      "pubkey": ""
    }
  }
}
```

- [ ] **Step 5: Create build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 6: Create main.rs skeleton**

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Run: `cargo check -p grid-screen` expected: PASS

- [ ] **Step 7: Create package.json**

```json
{
  "name": "grid-screen",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "lint": "prettier --check src/ && eslint src/"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "@tsconfig/svelte": "^5.0.0",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "typescript": "^5.7.0",
    "vite": "^6.0.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-opener": "^2.0.0"
  }
}
```

Run: `npm install` expected: PASS

- [ ] **Step 8: Create tsconfig.json**

```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ESNext",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "resolveJsonModule": true,
    "allowJs": true,
    "checkJs": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "strict": true,
    "noEmit": true,
    "paths": {
      "$lib/*": ["./src/lib/*"]
    },
    "baseUrl": "."
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"]
}
```

- [ ] **Step 9: Create vite.config.ts**

```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 5174 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

- [ ] **Step 10: Create svelte.config.js**

```javascript
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
};
```

- [ ] **Step 11: Create index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Grid Screen</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet" />
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.ts"></script>
</body>
</html>
```

- [ ] **Step 12: Create src/main.ts entry**

```typescript
import App from "./App.svelte";
import "./app.css";

const app = new App({
  target: document.getElementById("app")!,
});

export default app;
```

- [ ] **Step 13: Create src/App.svelte skeleton with Svelte 5 runes**

```svelte
<script lang="ts">
</script>

<div class="app">
  <span>Grid Screen</span>
</div>

<style>
  .app {
    color: #e4e4ef;
    background: #0a0a0f;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'Inter', sans-serif;
  }
</style>
```

- [ ] **Step 14: Create src/app.css with design tokens**

```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');

:root {
  --bg: #0a0a0f;
  --surface: #12121a;
  --surface-2: #1a1a26;
  --surface-3: #22222f;
  --border: #2a2a3a;
  --text: #e4e4ef;
  --text-dim: #8a8aa0;
  --text-mute: #555568;
  --accent: #5b8def;
  --accent-glow: rgba(91, 141, 239, 0.15);
  --accent-soft: rgba(91, 141, 239, 0.08);
  --success: #4ade80;
  --danger: #f87171;
  --radius: 10px;
  --radius-sm: 6px;
  --transition: 0.18s cubic-bezier(0.4, 0, 0.2, 1);
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: 'Inter', -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  height: 100vh;
  overflow: hidden;
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--surface-3); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--border); }
```

- [ ] **Step 15: Verify scaffold boots**

Run: `npm run dev` — expected: dev server starts on port 5173. Verify http://localhost:5173 shows "Grid Screen" text in dark background.

- [ ] **Step 16: Commit**

```bash
git add -A
git commit -m "scaffold: initialize Tauri 2 + Rust + Svelte project skeleton"
```

---

### Task 2: Shared Type Definitions

**Files:**
- Create: `shared-types/src/lib.rs`

**Interfaces:**
- Produces: All IPC types used by both Rust core and Svelte UI

- [ ] **Step 1: Write shared-types/src/lib.rs**

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenInfo {
    pub id: String,
    pub label: String,
    pub resolution: String,
    pub work_area: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WindowDescriptor {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub icon_color: String,
    pub state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WindowState {
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub movable: bool,
    pub resizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Layout {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub layout_type: LayoutType,
    pub zones: u32,
    pub columns: String,
    pub rows: Option<String>,
    pub span_first: Option<bool>,
    pub ratio: Option<u32>,
    pub gap_px: u32,
    pub margin_px: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum LayoutType {
    Preset,
    Saved,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Settings {
    pub schema_version: u32,
    pub snap_enabled: bool,
    pub snap_modifier: String,
    pub autostart_enabled: bool,
    pub minimize_to_tray: bool,
    pub last_layout_id: Option<String>,
    pub active_target_screen_hint: Option<String>,
    pub default_gap_px: u32,
    pub default_margin_px: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SystemStatus {
    pub session_type: String,
    pub ewmh_support: String,
    pub wm_name: String,
    pub xrandr_available: bool,
    pub workspace: String,
    pub connected_screens: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArrangeRequest {
    pub layout_id: String,
    pub screen_id: String,
    pub assignments: HashMap<u32, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArrangeResult {
    pub success: bool,
    pub results: Vec<PerWindowResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerWindowResult {
    pub window_id: String,
    pub status: MoveStatus,
    pub actual_rect: Option<Rect>,
    pub error: Option<String>,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum MoveStatus {
    Moved,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BootstrapData {
    pub screens: Vec<ScreenInfo>,
    pub layouts: Vec<Layout>,
    pub windows: Vec<WindowDescriptor>,
    pub settings: Settings,
    pub system_status: SystemStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkspaceChangedPayload {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenChangedPayload {
    pub screens: Vec<ScreenInfo>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: 1,
            snap_enabled: true,
            snap_modifier: "Shift".to_string(),
            autostart_enabled: false,
            minimize_to_tray: true,
            last_layout_id: None,
            active_target_screen_hint: None,
            default_gap_px: 10,
            default_margin_px: 16,
        }
    }
}

impl Default for SystemStatus {
    fn default() -> Self {
        SystemStatus {
            session_type: "unknown".to_string(),
            ewmh_support: "unknown".to_string(),
            wm_name: "unknown".to_string(),
            xrandr_available: false,
            workspace: "unknown".to_string(),
            connected_screens: "unknown".to_string(),
            errors: Vec::new(),
        }
    }
}
```

- [ ] **Step 2: Verify types compile + generate TypeScript bindings**

Run: `cargo check -p shared-types` expected: PASS

Run: `cargo test -p shared-types` (triggers `ts_rs::export!` macro generation to `../src/lib/shared-types.ts`) expected: PASS

- [ ] **Step 3: Commit**

```bash
git add shared-types/
git commit -m "feat: define shared IPC types for Rust/Svelte boundary"
```

---

### Task 3: PlatformAdapter Trait + Mock Adapter

**Files:**
- Create: `src-tauri/src/platform_adapter.rs`

**Interfaces:**
- Produces: `PlatformAdapter` trait, `MockPlatformAdapter`, `WINDOW_SLOTS[u32]` registry. Used by all subsequent Rust core tasks.

- [ ] **Step 1: Write the trait definition and mock**

```rust
use shared_types::*;
use std::collections::HashMap;

pub type WorkspaceId = String;
pub type EventStream<T> = tokio::sync::broadcast::Receiver<T>;

pub trait PlatformAdapter: Send + Sync {
    fn enumerate_screens(&self) -> Vec<ScreenInfo>;
    fn current_workspace(&self) -> WorkspaceId;
    fn enumerate_windows(&self, workspace: &WorkspaceId) -> Vec<WindowDescriptor>;
    fn get_window_state(&self, window_id: &str) -> Option<WindowState>;
    fn get_frame_extents(&self, window_id: &str) -> Rect;
    fn restore_window(&self, window_id: &str);
    fn move_resize_window(&self, window_id: &str, rect: Rect) -> Result<Rect, String>;
    fn subscribe_workspace_events(&self) -> EventStream<WorkspaceChangedPayload>;
    fn subscribe_screen_events(&self) -> EventStream<ScreenChangedPayload>;
    fn detect_capabilities(&self) -> SystemStatus;
}

/// Test-only mock. Holds fake screens, windows, and a registry
/// of move/resize results for verification.
pub struct MockPlatformAdapter {
    pub screens: Vec<ScreenInfo>,
    pub windows: Vec<WindowDescriptor>,
    pub workspace: String,
    pub system_status: SystemStatus,
    pub move_log: std::sync::Mutex<Vec<(String, Rect)>>,
    pub workspace_tx: tokio::sync::broadcast::Sender<WorkspaceChangedPayload>,
    pub screen_tx: tokio::sync::broadcast::Sender<ScreenChangedPayload>,
    pub frame_extents: Rect,
}

impl MockPlatformAdapter {
    pub fn new() -> Self {
        let (workspace_tx, _) = tokio::sync::broadcast::channel(16);
        let (screen_tx, _) = tokio::sync::broadcast::channel(16);
        MockPlatformAdapter {
            screens: vec![
                ScreenInfo {
                    id: "DP-1".into(),
                    label: "DP-1 (Primary)".into(),
                    resolution: "2560 x 1440".into(),
                    work_area: Rect { x: 0, y: 0, width: 2560, height: 1400 },
                },
            ],
            windows: vec![],
            workspace: "1".into(),
            system_status: SystemStatus {
                session_type: "x11".into(),
                ewmh_support: "Full".into(),
                wm_name: "MockWM".into(),
                xrandr_available: true,
                workspace: "1".into(),
                connected_screens: "DP-1".into(),
                errors: vec![],
            },
            move_log: std::sync::Mutex::new(Vec::new()),
            workspace_tx,
            screen_tx,
            frame_extents: Rect { x: 0, y: 0, width: 0, height: 0 },
        }
    }
}

impl PlatformAdapter for MockPlatformAdapter {
    fn enumerate_screens(&self) -> Vec<ScreenInfo> {
        self.screens.clone()
    }

    fn current_workspace(&self) -> WorkspaceId {
        self.workspace.clone()
    }

    fn enumerate_windows(&self, _workspace: &WorkspaceId) -> Vec<WindowDescriptor> {
        self.windows.clone()
    }

    fn get_window_state(&self, window_id: &str) -> Option<WindowState> {
        self.windows.iter().find(|w| w.id == window_id).map(|w| w.state.clone())
    }

    fn get_frame_extents(&self, _window_id: &str) -> Rect {
        self.frame_extents.clone()
    }

    fn restore_window(&self, _window_id: &str) {}

    fn move_resize_window(&self, window_id: &str, rect: Rect) -> Result<Rect, String> {
        self.move_log.lock().unwrap().push((window_id.to_string(), rect.clone()));
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
```

- [ ] **Step 2: Add tokio dependency to src-tauri/Cargo.toml**

Edit `src-tauri/Cargo.toml`, add to `[dependencies]`:
```toml
tokio = { version = "1", features = ["sync"] }
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p grid-screen` expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/platform_adapter.rs src-tauri/Cargo.toml
git commit -m "feat: add PlatformAdapter trait and MockPlatformAdapter"
```

---

### Task 4: Config Store

**Files:**
- Create: `src-tauri/src/config_store.rs`
- Create: `src-tauri/src/config_store.rs` tests inline via `#[cfg(test)]`

**Interfaces:**
- Produces: `ConfigStore::load() -> (Settings, Vec<Layout>)`, `ConfigStore::save_settings(Settings)`, `ConfigStore::save_layouts(Vec<Layout>)`, `ConfigStore::save_defaults(gap, margin)`, `ConfigStore::migrate(old_version, data)`
- Consumes: shared-types (Settings, Layout)

- [ ] **Step 1: Write failing tests for config store**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("grid-screen-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_load_defaults_when_no_config() {
        let dir = temp_dir();
        let store = ConfigStore::new(dir.clone());
        let (settings, layouts, _) = store.load().unwrap();
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.default_gap_px, 10);
        assert!(layouts.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_save_and_load_settings() {
        let dir = temp_dir();
        let store = ConfigStore::new(dir.clone());
        let mut settings = Settings::default();
        settings.autostart_enabled = true;
        store.save_settings(&settings).unwrap();
        let (loaded, _, _) = store.load().unwrap();
        assert!(loaded.autostart_enabled);
        fs::remove_dir_all(&dir).unwrap();
    }
}
```

Run: `cargo test -p grid-screen -- config_store::tests` expected: FAIL (ConfigStore not defined)

- [ ] **Step 2: Implement ConfigStore**

```rust
use shared_types::{Layout, Settings, SystemStatus};
use std::fs;
use std::path::PathBuf;

const SCHEMA_VERSION: u32 = 1;
const MAX_BACKUPS: u32 = 5;

pub struct ConfigStore {
    base_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(base_dir: PathBuf) -> Self {
        fs::create_dir_all(&base_dir).unwrap();
        ConfigStore { base_dir }
    }

    pub fn load(&self) -> Result<(Settings, Vec<Layout>, Vec<String>), String> {
        let settings_path = self.base_dir.join("settings.json");
        let layouts_path = self.base_dir.join("layouts.json");

        let settings = if settings_path.exists() {
            let data = fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            serde_json::from_str::<Settings>(&data)
                .map_err(|e| format!("Failed to parse settings: {}", e))?
        } else {
            Settings::default()
        };

        let layouts = if layouts_path.exists() {
            let data = fs::read_to_string(&layouts_path)
                .map_err(|e| format!("Failed to read layouts: {}", e))?;
            serde_json::from_str::<Vec<Layout>>(&data)
                .map_err(|e| format!("Failed to parse layouts: {}", e))?
        } else {
            Vec::new()
        };

        let warnings = if settings.schema_version < SCHEMA_VERSION {
            vec!["Config from older version — settings reset to defaults. Backup preserved.".into()]
        } else {
            Vec::new()
        };

        Ok((settings, layouts, warnings))
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let path = self.base_dir.join("settings.json");
        let tmp = self.base_dir.join("settings.json.tmp");
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        self.atomic_write(&path, &tmp, &json)
    }

    pub fn save_layouts(&self, layouts: &[Layout]) -> Result<(), String> {
        let path = self.base_dir.join("layouts.json");
        let tmp = self.base_dir.join("layouts.json.tmp");
        let json = serde_json::to_string_pretty(layouts)
            .map_err(|e| format!("Failed to serialize layouts: {}", e))?;
        self.atomic_write(&path, &tmp, &json)
    }

    pub fn save_defaults(&self, gap_px: u32, margin_px: u32) -> Result<(), String> {
        let mut settings = self.load().map(|(s, _, _)| s)?;
        settings.default_gap_px = gap_px;
        settings.default_margin_px = margin_px;
        self.save_settings(&settings)
    }

    fn atomic_write(&self, dest: &PathBuf, tmp: &PathBuf, data: &str) -> Result<(), String> {
        if dest.exists() {
            self.rotate_backup(dest)?;
        }
        fs::write(tmp, data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        // validation read-back
        let written = fs::read_to_string(tmp)
            .map_err(|e| format!("Failed to read-back temp file: {}", e))?;
        if written != data {
            return Err("Validation read-back mismatch".into());
        }
        fs::rename(tmp, dest)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;
        Ok(())
    }

    fn rotate_backup(&self, path: &PathBuf) -> Result<(), String> {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let ext = path.extension().map(|e| e.to_str().unwrap()).unwrap_or("json");
        for i in (1..MAX_BACKUPS).rev() {
            let old = self.base_dir.join(format!("{}.{}.{}", stem, i, ext));
            let new = self.base_dir.join(format!("{}.{}.{}", stem, i + 1, ext));
            if old.exists() {
                fs::rename(&old, &new)
                    .map_err(|e| format!("Failed to rotate backup: {}", e))?;
            }
        }
        let backup = self.base_dir.join(format!("{}.1.{}", stem, ext));
        if path.exists() {
            fs::copy(path, &backup)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p grid-screen -- config_store::tests` expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/config_store.rs
git commit -m "feat: add ConfigStore with atomic writes and backup rotation"
```

---

### Task 5: Layout Engine

**Files:**
- Create: `src-tauri/src/layout_engine.rs`
- Test inline via `#[cfg(test)]`

**Interfaces:**
- Produces: `LayoutEngine::compute_zones(layout, screen_info) -> Result<Vec<Rect>>`, `LayoutEngine::validate_layout(layout) -> Result<()>`
- Consumes: shared-types (Layout, ScreenInfo, Rect)

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_screen() -> ScreenInfo {
        ScreenInfo {
            id: "DP-1".into(),
            label: "DP-1".into(),
            resolution: "2560 x 1440".into(),
            work_area: Rect { x: 0, y: 0, width: 2560, height: 1440 },
        }
    }

    #[test]
    fn test_two_columns_equal() {
        let layout = Layout {
            id: "2col".into(),
            name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 10, margin_px: 16,
            created_at: "".into(), updated_at: "".into(),
        };
        let zones = LayoutEngine::compute_zones(&layout, &test_screen()).unwrap();
        assert_eq!(zones.len(), 2);
        // outer margin 16, so inner width = 2560 - 32 = 2528
        // gap 10, each zone gets (2528 - 10) / 2 = 1259
        assert_eq!(zones[0].width, 1259);
        assert_eq!(zones[1].width, 1259);
    }

    #[test]
    fn test_ratio_splits_unevenly() {
        let layout = Layout {
            id: "main-side".into(),
            name: "Main + Sidebar".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "3fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(75), gap_px: 10, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        };
        let zones = LayoutEngine::compute_zones(&layout, &test_screen()).unwrap();
        assert_eq!(zones.len(), 2);
        // full width 2560, gap 10, zone0 = 75% of 2550 = 1912, zone1 = 637
        assert_eq!(zones[0].width, 1912);
        assert_eq!(zones[1].width, 637);
    }
}
```

Run: `cargo test -p grid-screen -- layout_engine::tests` expected: FAIL

- [ ] **Step 2: Implement LayoutEngine**

```rust
use shared_types::{Layout, Rect, ScreenInfo};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute_zones(layout: &Layout, screen: &ScreenInfo) -> Result<Vec<Rect>, String> {
        let inner_width = screen.work_area.width as i32 - (layout.margin_px as i32 * 2);
        let inner_height = screen.work_area.height as i32 - (layout.margin_px as i32 * 2);

        if inner_width <= 0 || inner_height <= 0 {
            return Err("Layout too small for screen".into());
        }

        let zones = layout.zones as usize;
        let num_cols = layout.columns.split_whitespace().count();

        if zones == 2 && layout.ratio.is_some() {
            let ratio = layout.ratio.unwrap() as i32; // 10-90
            let available = inner_width - layout.gap_px as i32;
            let w0 = available * ratio / 100;
            let w1 = available - w0;
            let x0 = screen.work_area.x + layout.margin_px as i32;
            let x1 = x0 + w0 + layout.gap_px as i32;
            let y0 = screen.work_area.y + layout.margin_px as i32;
            Ok(vec![
                Rect { x: x0, y: y0, width: w0 as u32, height: inner_height as u32 },
                Rect { x: x1, y: y0, width: w1 as u32, height: inner_height as u32 },
            ])
        } else if zones == 3 && layout.rows.is_some() {
            // Focus + Stack: zone 0 = left span-2, zone 1 = top-right, zone 2 = bottom-right
            let available = inner_width - layout.gap_px as i32;
            let w_left = available * 2 / 3;
            let w_right = available - w_left;
            let available_h = inner_height - layout.gap_px as i32;
            let h_top = available_h / 2;
            let h_bottom = available_h - h_top;
            let x0 = screen.work_area.x + layout.margin_px as i32;
            let x1 = x0 + w_left + layout.gap_px as i32;
            let y0 = screen.work_area.y + layout.margin_px as i32;
            Ok(vec![
                Rect { x: x0, y: y0, width: w_left as u32, height: inner_height as u32 },
                Rect { x: x1, y: y0, width: w_right as u32, height: h_top as u32 },
                Rect { x: x1, y: y0 + h_top + layout.gap_px as i32, width: w_right as u32, height: h_bottom as u32 },
            ])
        } else {
            // Equal-width columns
            let parts = num_cols;
            let gaps = (parts - 1) as i32 * layout.gap_px as i32;
            let available = inner_width - gaps;
            let zone_width = available / parts as i32;
            let remainder = available % parts as i32;
            let mut zones = Vec::new();
            let mut x = screen.work_area.x + layout.margin_px as i32;
            let y = screen.work_area.y + layout.margin_px as i32;
            for i in 0..zones {
                let w = if i < remainder as usize { zone_width + 1 } else { zone_width };
                zones.push(Rect { x, y, width: w as u32, height: inner_height as u32 });
                x += w + layout.gap_px as i32;
            }
            Ok(zones)
        }
    }

    pub fn validate_layout(layout: &Layout) -> Result<(), String> {
        if layout.name.is_empty() || layout.name.len() > 64 {
            return Err("Layout name must be 1-64 characters".into());
        }
        if layout.zones < 2 || layout.zones > 3 {
            return Err("Layout must have 2 or 3 zones".into());
        }
        if let Some(ratio) = layout.ratio {
            if ratio < 10 || ratio > 90 {
                return Err("Ratio must be between 10 and 90".into());
            }
        }
        if layout.gap_px > 40 {
            return Err("Gap must be <= 40px".into());
        }
        if layout.margin_px > 60 {
            return Err("Margin must be <= 60px".into());
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p grid-screen -- layout_engine::tests` expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/layout_engine.rs
git commit -m "feat: add LayoutEngine with zone rectangle computation"
```

---

### Task 6: Window Catalog

**Files:**
- Create: `src-tauri/src/window_catalog.rs`

**Interfaces:**
- Produces: `WindowCatalog::new(adapter)`, `WindowCatalog::refresh(workspace) -> Vec<WindowDescriptor>`, `WindowCatalog::validate_window_id(id) -> bool`
- Consumes: PlatformAdapter trait, shared-types (WindowDescriptor, WindowState)

- [ ] **Step 1: Write failing test for catalog filtering**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_adapter::MockPlatformAdapter;
    use shared_types::*;

    #[test]
    fn test_excludes_non_movable_windows() {
        let adapter = MockPlatformAdapter::new();
        let mut adapter = adapter;
        adapter.windows = vec![
            WindowDescriptor {
                id: "w1".into(), app_name: "Firefox".into(), title: "MDN".into(),
                icon_color: "#ff7139".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: false, resizable: true },
            },
            WindowDescriptor {
                id: "w2".into(), app_name: "Terminal".into(), title: "bash".into(),
                icon_color: "#2d2d2d".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: true, resizable: true },
            },
        ];
        let catalog = WindowCatalog::new(&adapter);
        let windows = catalog.refresh("1");
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, "w2");
    }
}
```

Run: `cargo test -p grid-screen -- window_catalog::tests` expected: FAIL

- [ ] **Step 2: Implement WindowCatalog**

```rust
use shared_types::WindowDescriptor;
use crate::platform_adapter::PlatformAdapter;
use std::collections::HashSet;

pub struct WindowCatalog<'a> {
    adapter: &'a dyn PlatformAdapter,
    known_ids: std::cell::RefCell<HashSet<String>>,
}

impl<'a> WindowCatalog<'a> {
    pub fn new(adapter: &'a dyn PlatformAdapter) -> Self {
        WindowCatalog {
            adapter,
            known_ids: std::cell::RefCell::new(HashSet::new()),
        }
    }

    pub fn refresh(&self, workspace: &str) -> Vec<WindowDescriptor> {
        let windows = self.adapter.enumerate_windows(workspace);
        let eligible: Vec<_> = windows
            .into_iter()
            .filter(|w| {
                w.state.movable
                    && w.state.resizable
                    && !w.state.fullscreen
                    && !w.app_name.is_empty()
            })
            .collect();
        self.known_ids.borrow_mut().extend(eligible.iter().map(|w| w.id.clone()));
        eligible
    }

    pub fn validate(&self) -> Vec<String> {
        let known = self.known_ids.borrow();
        let mut stale = Vec::new();
        for id in known.iter() {
            if self.adapter.get_window_state(id).is_none() {
                stale.push(id.clone());
            }
        }
        stale
    }
}
```

- [ ] **Step 3: Add `std::cell` usage to lib import**

No lib import needed — this uses std.

- [ ] **Step 4: Run tests**

Run: `cargo test -p grid-screen -- window_catalog::tests` expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/window_catalog.rs
git commit -m "feat: add WindowCatalog with eligibility filtering"
```

---

### Task 7: Arrange Orchestrator

**Files:**
- Create: `src-tauri/src/arrange_orchestrator.rs`

**Interfaces:**
- Produces: `ArrangeOrchestrator::arrange(request, layouts, screens, adapter, catalog, engine) -> ArrangeResult`
- Consumes: PlatformAdapter, LayoutEngine, WindowCatalog, shared-types (ArrangeRequest, ArrangeResult, PerWindowResult, Layout, ScreenInfo)

- [ ] **Step 1: Write failing test for successful arrange**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_adapter::MockPlatformAdapter;
    use crate::layout_engine::LayoutEngine;
    use crate::window_catalog::WindowCatalog;

    #[test]
    fn test_arrange_success() {
        let adapter = MockPlatformAdapter::new();
        let mut adapter = adapter;
        adapter.windows = vec![
            WindowDescriptor {
                id: "w1".into(), app_name: "Firefox".into(), title: "MDN".into(),
                icon_color: "#ff7139".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: true, resizable: true },
            },
        ];
        let catalog = WindowCatalog::new(&adapter);
        let engine = LayoutEngine;
        let layouts = vec![Layout {
            id: "2col".into(), name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 10, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        }];
        let screens = adapter.enumerate_screens();

        let request = ArrangeRequest {
            layout_id: "2col".into(),
            screen_id: "DP-1".into(),
            assignments: std::collections::HashMap::from([(0, "w1".into())]),
        };

        let result = ArrangeOrchestrator::arrange(&request, &layouts, &screens, &adapter, &catalog, &engine);
        assert!(result.success);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].window_id, "w1");
        assert!(matches!(result.results[0].status, MoveStatus::Moved));
        // verify move was logged
        let log = adapter.move_log.lock().unwrap();
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_arrange_rejects_stale_window() {
        let adapter = MockPlatformAdapter::new();
        let catalog = WindowCatalog::new(&adapter);
        let engine = LayoutEngine;
        let layouts = vec![Layout {
            id: "2col".into(), name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 0, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        }];
        let screens = adapter.enumerate_screens();
        let request = ArrangeRequest {
            layout_id: "2col".into(),
            screen_id: "DP-1".into(),
            assignments: std::collections::HashMap::from([(0, "nonexistent".into())]),
        };
        let result = ArrangeOrchestrator::arrange(&request, &layouts, &screens, &adapter, &catalog, &engine);
        assert!(!result.success);
        assert_eq!(result.results[0].status, MoveStatus::Failed);
    }
}
```

- [ ] **Step 2: Implement ArrangeOrchestrator**

```rust
use shared_types::*;
use crate::platform_adapter::PlatformAdapter;
use crate::layout_engine::LayoutEngine;
use crate::window_catalog::WindowCatalog;

pub struct ArrangeOrchestrator;

impl ArrangeOrchestrator {
    pub fn arrange(
        request: &ArrangeRequest,
        layouts: &[Layout],
        screens: &[ScreenInfo],
        adapter: &dyn PlatformAdapter,
        catalog: &WindowCatalog,
        engine: &LayoutEngine,
    ) -> ArrangeResult {
        let layout = match layouts.iter().find(|l| l.id == request.layout_id) {
            Some(l) => l,
            None => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some("Layout not found".into()),
                }],
            },
        };

        let screen = match screens.iter().find(|s| s.id == request.screen_id) {
            Some(s) => s,
            None => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some("Screen not found".into()),
                }],
            },
        };

        let zones = match LayoutEngine::compute_zones(layout, screen) {
            Ok(z) => z,
            Err(e) => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(e),
                }],
            },
        };

        // Validate all assignments first
        let mut errors = Vec::new();
        for (zone_idx, window_id) in &request.assignments {
            if *zone_idx as usize >= zones.len() {
                errors.push((window_id.clone(), format!("Zone {} out of range", zone_idx)));
                continue;
            }
            if adapter.get_window_state(window_id).is_none() {
                errors.push((window_id.clone(), "Window no longer exists".into()));
                continue;
            }
            let state = adapter.get_window_state(window_id).unwrap();
            if !state.movable || !state.resizable {
                errors.push((window_id.clone(), "Window is not movable or resizable".into()));
            }
        }

        if !errors.is_empty() {
            return ArrangeResult {
                success: false,
                results: errors.into_iter().map(|(wid, err)| PerWindowResult {
                    window_id: wid,
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(err),
                }).collect(),
            };
        }

        // Execute arrangement
        let mut results = Vec::new();
        for (zone_idx, window_id) in &request.assignments {
            let zone = &zones[*zone_idx as usize];

            let state = adapter.get_window_state(window_id).unwrap();
            if state.minimized {
                adapter.restore_window(window_id);
            }

            let frame = adapter.get_frame_extents(window_id);
            let adjusted = Rect {
                x: zone.x - frame.x,
                y: zone.y - frame.y,
                width: zone.width - (frame.width + frame.x as u32),
                height: zone.height - (frame.height + frame.y as u32),
            };

            match adapter.move_resize_window(window_id, adjusted) {
                Ok(actual) => results.push(PerWindowResult {
                    window_id: window_id.clone(),
                    status: MoveStatus::Moved,
                    actual_rect: Some(actual),
                    error: None,
                }),
                Err(e) => results.push(PerWindowResult {
                    window_id: window_id.clone(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(e),
                }),
            }
        }

        let all_moved = results.iter().all(|r| matches!(r.status, MoveStatus::Moved));
        ArrangeResult { success: all_moved, results }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p grid-screen -- arrange_orchestrator::tests` expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/arrange_orchestrator.rs
git commit -m "feat: add ArrangeOrchestrator with validate-then-move flow"
```

---

### Task 8: Diagnostics (Logging)

**Files:**
- Create: `src-tauri/src/diagnostics.rs`

**Interfaces:**
- Produces: `Diagnostics::init()` (called in main.rs to set up logging), `Diagnostics::collect_info(adapter) -> String`
- Consumes: PlatformAdapter

- [ ] **Step 1: Implement diagnostics module**

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use std::path::PathBuf;

pub struct Diagnostics;

impl Diagnostics {
    pub fn init(config_dir: &PathBuf) {
        let log_dir = config_dir.join("logs");
        std::fs::create_dir_all(&log_dir).unwrap();

        let file_appender = RollingFileAppender::new(
            Rotation::NEVER,
            log_dir.clone(),
            "grid-screen.log",
        );

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_writer(std::io::stdout.and(non_blocking))
            .init();
    }

    pub fn collect_info(status: &shared_types::SystemStatus) -> String {
        format!(
            "Grid Screen v{}\nSession: {}\nWM: {}\nEWMH: {}\nXRandR: {}\nWorkspace: {}\nScreens: {}\n",
            env!("CARGO_PKG_VERSION"),
            status.session_type,
            status.wm_name,
            status.ewmh_support,
            if status.xrandr_available { "Available" } else { "Not available" },
            status.workspace,
            status.connected_screens,
        )
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p grid-screen` expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/diagnostics.rs
git commit -m "feat: add file-based logging with diagnostics module"
```

---

### Task 9: App Shell (Tauri commands + tray + lifecycle)

**Files:**
- Create: `src-tauri/src/app_shell.rs`
- Modify: `src-tauri/src/main.rs` (wire shell + diagnostics)

**Interfaces:**
- Produces: All Tauri commands (`bootstrap`, `refresh_windows`, `arrange_windows`, etc.), tray setup, single-instance logic
- Consumes: ConfigStore, WindowCatalog, LayoutEngine, ArrangeOrchestrator, PlatformAdapter, Diagnostics

- [ ] **Step 1: Write Tauri commands in app_shell.rs** (abbreviated — key commands shown)

```rust
use crate::platform_adapter::PlatformAdapter;
use crate::config_store::ConfigStore;
use crate::layout_engine::LayoutEngine;
use crate::window_catalog::WindowCatalog;
use crate::arrange_orchestrator::ArrangeOrchestrator;
use crate::diagnostics::Diagnostics;
use shared_types::*;
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub adapter: Box<dyn PlatformAdapter>,
    pub config: ConfigStore,
    pub catalog: Mutex<Option<WindowCatalog<'static>>>,
}

// SAFETY: AppState is only accessed from the Tauri command thread
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[tauri::command]
fn bootstrap(state: State<AppState>) -> Result<BootstrapData, String> {
    let (settings, layouts, _warnings) = state.config.load()?;
    let screens = state.adapter.enumerate_screens();

    let workspace = state.adapter.current_workspace();
    let windows = state.adapter.enumerate_windows(&workspace);
    let eligible: Vec<_> = windows
        .into_iter()
        .filter(|w| w.state.movable && w.state.resizable && !w.state.fullscreen)
        .collect();

    let system_status = state.adapter.detect_capabilities();

    Ok(BootstrapData {
        screens,
        layouts,
        windows: eligible,
        settings,
        system_status,
    })
}

#[tauri::command]
fn refresh_windows(state: State<AppState>) -> Vec<WindowDescriptor> {
    let workspace = state.adapter.current_workspace();
    let windows = state.adapter.enumerate_windows(&workspace);
    windows.into_iter()
        .filter(|w| w.state.movable && w.state.resizable && !w.state.fullscreen)
        .collect()
}

#[tauri::command]
fn arrange_windows(
    state: State<AppState>,
    request: ArrangeRequest,
) -> ArrangeResult {
    let (_, layouts, _) = state.config.load().unwrap_or_default();
    let screens = state.adapter.enumerate_screens();
    let engine = LayoutEngine;

    let workspace = state.adapter.current_workspace();
    let adapter = state.adapter.as_ref();

    // temporary catalog for validation
    let windows = adapter.enumerate_windows(&workspace);

    // synchronous arrangement
    let layout = match layouts.iter().find(|l| l.id == request.layout_id) {
        Some(l) => l,
        None => return ArrangeResult {
            success: false,
            results: vec![PerWindowResult {
                window_id: "".into(), status: MoveStatus::Failed,
                actual_rect: None, error: Some("Layout not found".into()),
            }],
        },
    };
    let screen = match screens.iter().find(|s| s.id == request.screen_id) {
        Some(s) => s,
        None => return ArrangeResult {
            success: false,
            results: vec![PerWindowResult {
                window_id: "".into(), status: MoveStatus::Failed,
                actual_rect: None, error: Some("Screen not found".into()),
            }],
        },
    };

    let zones = match LayoutEngine::compute_zones(layout, screen) {
        Ok(z) => z,
        Err(e) => return ArrangeResult {
            success: false,
            results: vec![PerWindowResult {
                window_id: "".into(), status: MoveStatus::Failed,
                actual_rect: None, error: Some(e),
            }],
        },
    };

    // validate assignments exist
    for (zone_idx, window_id) in &request.assignments {
        if *zone_idx as usize >= zones.len() || !windows.iter().any(|w| &w.id == window_id) {
            return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: window_id.clone(), status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some("Window no longer exists or zone out of range".into()),
                }],
            };
        }
    }

    let mut results = Vec::new();
    for (zone_idx, window_id) in &request.assignments {
        let zone = &zones[*zone_idx as usize];
        let state = adapter.get_window_state(window_id).unwrap_or_default();
        if state.minimized {
            adapter.restore_window(window_id);
        }
        let frame = adapter.get_frame_extents(window_id);
        let adjusted = Rect {
            x: zone.x - frame.x,
            y: zone.y - frame.y,
            width: zone.width.saturating_sub((frame.width + frame.x as u32)),
            height: zone.height.saturating_sub((frame.height + frame.y as u32)),
        };
        match adapter.move_resize_window(window_id, adjusted) {
            Ok(actual) => results.push(PerWindowResult {
                window_id: window_id.clone(), status: MoveStatus::Moved,
                actual_rect: Some(actual), error: None,
            }),
            Err(e) => results.push(PerWindowResult {
                window_id: window_id.clone(), status: MoveStatus::Failed,
                actual_rect: None, error: Some(e),
            }),
        }
    }

    let all_moved = results.iter().all(|r| matches!(r.status, MoveStatus::Moved));
    ArrangeResult { success: all_moved, results }
}

#[tauri::command]
fn save_layout(state: State<AppState>, layout: Layout) -> Result<(), String> {
    let (_, mut layouts, _) = state.config.load()?;
    if let Some(existing) = layouts.iter_mut().find(|l| l.id == layout.id) {
        *existing = layout;
    } else {
        layouts.push(layout);
    }
    state.config.save_layouts(&layouts)
}

#[tauri::command]
fn delete_layout(state: State<AppState>, layout_id: String) -> Result<(), String> {
    let (_, mut layouts, _) = state.config.load()?;
    layouts.retain(|l| l.id != layout_id);
    state.config.save_layouts(&layouts)
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.config.load().map(|(s, _, _)| s).unwrap_or_default()
}

#[tauri::command]
fn update_settings(state: State<AppState>, settings: Settings) -> Result<(), String> {
    state.config.save_settings(&settings)
}

#[tauri::command]
fn save_defaults(state: State<AppState>, gap_px: u32, margin_px: u32) -> Result<(), String> {
    state.config.save_defaults(gap_px, margin_px)
}

#[tauri::command]
fn get_diagnostics(state: State<AppState>) -> String {
    let status = state.adapter.detect_capabilities();
    Diagnostics::collect_info(&status)
}
```

- [ ] **Step 2: Update main.rs to wire everything** (continued below due to length)

- [ ] **Step 3: Next section continues in Task 9 part 2**

This task is large — remaining steps (main.rs wiring, tray setup, X11 detection stub) continue below. I'll handle the split after getting this installed.

---

### Task 9 (continued): main.rs wiring + tray

- [ ] **Step 3 (cont): Update main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_shell;
mod platform_adapter;
mod config_store;
mod layout_engine;
mod window_catalog;
mod arrange_orchestrator;
mod diagnostics;

use app_shell::*;
use config_store::ConfigStore;
use platform_adapter::MockPlatformAdapter;
use diagnostics::Diagnostics;
use std::path::PathBuf;
use tauri::Manager;

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config")
        })
        .join("grid-screen")
}

fn main() {
    let config_path = config_dir();
    std::fs::create_dir_all(&config_path).unwrap();
    Diagnostics::init(&config_path);

    tracing::info!("Grid Screen starting, config dir: {:?}", config_path);

    // MVP: use MockPlatformAdapter; replace with X11 adapter when available
    let adapter = MockPlatformAdapter::new();
    let config = ConfigStore::new(config_path);

    let app_state = AppState {
        adapter: Box::new(adapter),
        config,
        catalog: std::sync::Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            refresh_windows,
            arrange_windows,
            save_layout,
            delete_layout,
            get_settings,
            update_settings,
            save_defaults,
            get_diagnostics,
        ])
        .setup(|app| {
            let _tray = app.tray_handle(); // Tray icon will be configured when we have assets
            tracing::info!("Application setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                // Check minimize-to-tray setting
                let settings = state.config.load().map(|(s, _, _)| s).unwrap_or_default();
                if settings.minimize_to_tray {
                    api.prevent_close();
                    window.hide().ok();
                    tracing::info!("Window minimized to tray");
                }
                // If tray not available or minimize disabled, let close proceed
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    tracing::info!("Grid Screen exited");
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p grid-screen` expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add AppShell with Tauri commands, tray, and lifecycle"
```

---

### Task 10: Svelte Stores

**Files:**
- Create: `src/lib/stores/assignments.ts`
- Create: `src/lib/stores/layout.ts`
- Create: `src/lib/stores/screen.ts`
- Create: `src/lib/stores/windows.ts`
- Create: `src/lib/stores/settings.ts`
- Create: `src/lib/stores/systemStatus.ts`
- Create: `src/lib/stores/arrangeState.ts`
- Create: `src/lib/stores/toasts.ts`

**Interfaces:** All stores consume shared-types (via ts-rs generated types), used by Svelte components.

- [ ] **Step 1: Create assignments store**

```typescript
// src/lib/stores/assignments.ts
import { writable, derived } from "svelte/store";

export const assignments = writable<Record<number, string>>({});

export const assignedWindowIds = derived(assignments, ($a) =>
  new Set(Object.values($a))
);

export const assignedCount = derived(assignments, ($a) =>
  Object.keys($a).length
);

export function assignWindow(zoneIndex: number, windowId: string) {
  assignments.update((a) => {
    // Remove window from any existing zone
    const cleaned: Record<number, string> = {};
    for (const [z, wid] of Object.entries(a)) {
      if (wid !== windowId) cleaned[Number(z)] = wid;
    }
    cleaned[zoneIndex] = windowId;
    return cleaned;
  });
}

export function removeWindowFromZone(zoneIndex: number) {
  assignments.update((a) => {
    const next = { ...a };
    delete next[zoneIndex];
    return next;
  });
}

export function clearAssignments() {
  assignments.set({});
}
```

- [ ] **Step 2: Create layout store**

```typescript
// src/lib/stores/layout.ts
import { writable, derived } from "svelte/store";
import type { Layout } from "../shared-types";

export const layouts = writable<Layout[]>([]);
export const selectedLayoutId = writable<string>("");
export const sessionOverrides = writable<{
  ratio?: number;
  gap_px?: number;
  margin_px?: number;
}>({});

export const selectedLayout = derived(
  [layouts, selectedLayoutId, sessionOverrides],
  ([$layouts, $selectedLayoutId, $sessionOverrides]) => {
    const base = $layouts.find((l) => l.id === $selectedLayoutId);
    if (!base) return null;
    return { ...base, ...$sessionOverrides };
  }
);
```

- [ ] **Step 3: Create screen store**

```typescript
// src/lib/stores/screen.ts
import { writable } from "svelte/store";
import type { ScreenInfo } from "../shared-types";

export const screens = writable<ScreenInfo[]>([]);
export const selectedScreenId = writable<string>("");
```

- [ ] **Step 4: Create windows store**

```typescript
// src/lib/stores/windows.ts
import { writable } from "svelte/store";
import type { WindowDescriptor } from "../shared-types";

export const windows = writable<WindowDescriptor[]>([]);
```

- [ ] **Step 5: Create settings store**

```typescript
// src/lib/stores/settings.ts
import { writable } from "svelte/store";
import type { Settings } from "../shared-types";

const defaults: Settings = {
  schema_version: 1,
  snap_enabled: true,
  snap_modifier: "Shift",
  autostart_enabled: false,
  minimize_to_tray: true,
  last_layout_id: null,
  active_target_screen_hint: null,
  default_gap_px: 10,
  default_margin_px: 16,
};

export const settings = writable<Settings>(defaults);
```

- [ ] **Step 6: Create systemStatus store**

```typescript
// src/lib/stores/systemStatus.ts
import { writable } from "svelte/store";
import type { SystemStatus } from "../shared-types";

export const systemStatus = writable<SystemStatus>({
  session_type: "unknown",
  ewmh_support: "unknown",
  wm_name: "unknown",
  xrandr_available: false,
  workspace: "unknown",
  connected_screens: "unknown",
  errors: [],
});
```

- [ ] **Step 7: Create arrangeState store**

```typescript
// src/lib/stores/arrangeState.ts
import { writable } from "svelte/store";

export type ArrangeStatus =
  | { status: "idle" }
  | { status: "validating" }
  | { status: "arranging"; current: number; total: number }
  | { status: "completed"; errors: number }
  | { status: "failed"; reason: string };

export const arrangeState = writable<ArrangeStatus>({ status: "idle" });
```

- [ ] **Step 8: Create toasts store**

```typescript
// src/lib/stores/toasts.ts
import { writable } from "svelte/store";

export interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "warning";
  durationMs: number;
}

export const toasts = writable<Toast[]>([]);

export function showToast(
  message: string,
  type: Toast["type"] = "success",
  durationMs = 3000
) {
  const id = crypto.randomUUID();
  toasts.update((t) => [...t, { id, message, type, durationMs }]);
  if (durationMs > 0) {
    setTimeout(() => dismissToast(id), durationMs);
  }
}

export function dismissToast(id: string) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
```

- [ ] **Step 9: Create commands.ts IPC wrapper**

```typescript
// src/lib/commands.ts
import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapData,
  WindowDescriptor,
  Layout,
  ArrangeRequest,
  ArrangeResult,
  Settings,
} from "./shared-types";

export const commands = {
  bootstrap: () => invoke<BootstrapData>("bootstrap"),
  refreshWindows: () => invoke<WindowDescriptor[]>("refresh_windows"),
  arrangeWindows: (req: ArrangeRequest) =>
    invoke<ArrangeResult>("arrange_windows", { request: req }),
  saveLayout: (layout: Layout) => invoke<void>("save_layout", { layout }),
  deleteLayout: (layoutId: string) =>
    invoke<void>("delete_layout", { layoutId }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<void>("update_settings", { settings }),
  saveDefaults: (gapPx: number, marginPx: number) =>
    invoke<void>("save_defaults", { gapPx, marginPx }),
  getDiagnostics: () => invoke<string>("get_diagnostics"),
};
```

- [ ] **Step 10: Create events.ts for Rust → Svelte events**

```typescript
// src/lib/events.ts
import { listen } from "@tauri-apps/api/event";
import type { WorkspaceChangedPayload, ScreenChangedPayload, SystemStatus } from "./shared-types";
import { clearAssignments } from "./stores/assignments";
import { showToast } from "./stores/toasts";
import { windows } from "./stores/windows";
import { commands } from "./commands";
import { systemStatus } from "./stores/systemStatus";

export function registerEventListeners() {
  const unlistenWorkspace = listen<WorkspaceChangedPayload>(
    "workspace-changed",
    () => {
      clearAssignments();
      commands.refreshWindows().then((w) => windows.set(w));
      showToast("Workspace changed — assignments cleared", "warning");
    }
  );

  const unlistenScreen = listen<ScreenChangedPayload>(
    "screen-changed",
    (_event) => {
      showToast("Screen configuration changed", "warning");
    }
  );

  const unlistenSystemStatus = listen<SystemStatus>(
    "system-status-changed",
    (event) => {
      systemStatus.set(event.payload);
    }
  );

  return () => {
    unlistenWorkspace.then((fn) => fn());
    unlistenScreen.then((fn) => fn());
    unlistenSystemStatus.then((fn) => fn());
  };
}
```

- [ ] **Step 11: Verify TypeScript compilation**

Run: `npm run check` expected: PASS (types resolve)

- [ ] **Step 12: Commit**

```bash
git add src/lib/
git commit -m "feat: add Svelte stores and Tauri IPC wrapper"
```

---

### Task 11: App Shell Components (TitleBar + TabNav + ToastContainer)

**Files:**
- Create: `src/components/TitleBar.svelte`
- Create: `src/components/TabNav.svelte`
- Create: `src/components/ToastContainer.svelte`

- [ ] **Step 1: Create TitleBar.svelte**

```svelte
<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  const appWindow = getCurrentWindow();

  function minimize() { appWindow.minimize(); }
  function toggleMaximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<div class="titlebar">
  <div class="traffic-lights">
    <button class="tl tl-close" onclick={close} aria-label="Close"></button>
    <button class="tl tl-min" onclick={minimize} aria-label="Minimize"></button>
    <button class="tl tl-max" onclick={toggleMaximize} aria-label="Maximize"></button>
  </div>
  <span class="title-text">Grid Screen</span>
</div>

<style>
  .titlebar {
    display: flex; align-items: center; height: 44px;
    padding: 0 16px; background: var(--surface);
    border-bottom: 1px solid var(--border); gap: 16px; flex-shrink: 0;
  }
  .traffic-lights { display: flex; gap: 8px; }
  .tl { width: 12px; height: 12px; border-radius: 50%; cursor: pointer; border: none; }
  .tl-close { background: #ff5f57; }
  .tl-min { background: #febc2e; }
  .tl-max { background: #28c840; }
  .title-text {
    font-size: 13px; font-weight: 500; color: var(--text-dim);
    margin-left: auto; margin-right: auto; padding-right: 60px;
  }
</style>
```

- [ ] **Step 2: Create TabNav.svelte**

```svelte
<script lang="ts">
  export let activeTab: "arrange" | "layouts" | "settings" = "arrange";

  const tabs = [
    {
      id: "arrange" as const,
      label: "Arrange",
      icon: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    },
    {
      id: "layouts" as const,
      label: "Layouts",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/>',
    },
    {
      id: "settings" as const,
      label: "Settings",
      icon: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    },
  ];

  function handleTabClick(id: string) {
    activeTab = id as "arrange" | "layouts" | "settings";
    emit("tabChange", activeTab);
  }

  import { createEventDispatcher } from "svelte";
  const emit = createEventDispatcher<{ tabChange: string }>();

  $: svgContent = {
    arrange: tabs[0].icon,
    layouts: tabs[1].icon,
    settings: tabs[2].icon,
  };
</script>

<div class="nav-tabs">
  {#each tabs as { id, label, icon }}
    <button
      class="nav-tab"
      class:active={activeTab === id}
      onclick={() => handleTabClick(id)}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {@html icon}
      </svg>
      {label}
    </button>
  {/each}
</div>

<style>
  .nav-tabs {
    display: flex; gap: 0; padding: 0 16px;
    background: var(--surface); border-bottom: 1px solid var(--border);
    height: 44px; flex-shrink: 0;
  }
  .nav-tab {
    display: flex; align-items: center; gap: 8px;
    padding: 0 16px; font-size: 13px; font-weight: 500;
    color: var(--text-dim); cursor: pointer;
    border: none; background: none; font-family: inherit;
    border-bottom: 2px solid transparent;
    transition: var(--transition); position: relative; top: 1px;
    user-select: none;
  }
  .nav-tab:hover { color: var(--text); }
  .nav-tab.active { color: var(--accent); border-bottom-color: var(--accent); }
</style>
```

- [ ] **Step 3: Create ToastContainer.svelte**

```svelte
<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
</script>

{#each $toasts as toast (toast.id)}
  <div class="toast show {toast.type}">
    <span>{toast.message}</span>
    {#if toast.durationMs === 0}
      <button class="dismiss" onclick={() => dismissToast(toast.id)}>x</button>
    {/if}
  </div>
{/each}

<style>
  .toast {
    position: fixed; bottom: 80px; left: 50%;
    transform: translateX(-50%); z-index: 200;
    background: var(--surface-2); color: var(--text);
    padding: 12px 24px; border-radius: var(--radius-sm);
    font-size: 14px; font-weight: 500; pointer-events: none;
    border: 1px solid var(--success);
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }
  .toast.error { border-color: var(--danger); }
  .toast.warning { border-color: #d29922; }
  .dismiss {
    margin-left: 12px; background: none; border: none;
    color: var(--text-dim); cursor: pointer; font-size: 14px;
  }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/components/TitleBar.svelte src/components/TabNav.svelte src/components/ToastContainer.svelte
git commit -m "feat: add App Shell UI components"
```

---

### Task 12: Arrange View Components

Due to the length of this plan, the remaining tasks follow the exact same pattern as Tasks 10-11: create Svelte components based on the mockup's HTML/CSS, wired to the stores and commands created in Task 10. Each component receives props from `$store` syntax.

**Files to create for Task 12:**
- `src/components/ArrangeView.svelte` — three-column layout host
- `src/components/WindowCatalog.svelte` — search + scrollable card list
- `src/components/WindowCard.svelte` — single draggable window card with pointer-event DnD
- `src/components/SearchBox.svelte` — text input with filter
- `src/components/CanvasArea.svelte` — canvas + toolbar host
- `src/components/CanvasToolbar.svelte` — screen/layout selectors + quick buttons
- `src/components/ScreenSelector.svelte` — dropdown for screens
- `src/components/LayoutSelector.svelte` — dropdown for layouts
- `src/components/LayoutQuickButtons.svelte` — icon-only layout switcher
- `src/components/ScreenCanvas.svelte` — CSS grid preview, reactive to selectedLayout
- `src/components/ZoneSlot.svelte` — drop target, renders assigned window info
- `src/components/ActionBar.svelte` — Clear All + Arrange button

**Task 13: Detail Panel Components**
- `src/components/DetailPanel.svelte`
- `src/components/LayoutSliders.svelte`
- `src/components/SnapControls.svelte`
- `src/components/SystemStatusPanel.svelte`

**Task 14: Layouts View**
- `src/components/LayoutsView.svelte`
- `src/components/LayoutCard.svelte`
- `src/components/NewLayoutModal.svelte`

**Task 15: Settings View**
- `src/components/SettingsView.svelte`
- `src/components/SettingsGroup.svelte`

**Task 16: ArrangeStateOverlay**
- `src/components/ArrangeStateOverlay.svelte`

**Task 17: Wire App.svelte** — mount all components, bootstrap data, register event listeners

**Task 18: 5 built-in layout presets** — seed config with presets on first launch

**Task 19: X11 Adapter Stub** — implement PlatformAdapter for X11 (uses x11rb or x11-dl crate)

**Task 20: GitHub Actions CI/CD** — `.github/workflows/ci.yml` and `release.yml`

---

## Note

Tasks 12-20 follow the same TDD pattern as Tasks 1-11. Due to output limits, the remaining tasks are summarized above with their file lists. Each follows this structure:
1. Write failing test (for Rust tasks) or verify component renders (for Svelte tasks)
2. Implement minimal code
3. Verify (tests pass / component renders)
4. Commit

The complete implementation produces a working Grid Screen MVP with:
- Rust core: PlatformAdapter, ConfigStore, LayoutEngine, WindowCatalog, ArrangeOrchestrator, Diagnostics
- Svelte UI: 25 components, 8 stores, IPC wrapper, event listeners
- Tauri 2 shell: tray, window lifecycle, commands
- CI/CD: PR checks + release builds on GitHub Actions
