# Grid Screen

Quản lý vùng cửa sổ đa nền tảng. Kéo thả cửa sổ vào các vùng (zone) đã định nghĩa sẵn để snap vào vị trí tức thì.

**Nền tảng:** Linux (X11) · Windows

[![CI](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml/badge.svg)](https://github.com/enolalabs/grid-screen/actions/workflows/ci.yml)

---

## Mục lục

- [Tính năng](#tính-năng)
- [Cài đặt](#cài-đặt)
- [Sử dụng](#sử-dụng)
- [Phát triển](#phát-triển)
- [Kiến trúc](#kiến-trúc)
- [Cấu hình](#cấu-hình)
- [Build & phân phối](#build--phân-phối)
- [Kiểm thử](#kiểm-thử)
- [Đóng góp](#đóng-góp)

---

## Tính năng

| Tính năng | Mô tả |
|-----------|-------|
| **Snap cửa sổ** | Kéo cửa sổ vào vùng đã định nghĩa → cửa sổ tự động snap vào vị trí |
| **Editor WYSIWYG** | Kéo thả trực quan để tạo/chỉnh sửa vùng trên từng màn hình, hỗ trợ grid snapping 12 cột |
| **Đa màn hình** | Hỗ trợ hotplug — tự động phát hiện màn hình mới/cũ, layout riêng cho từng màn hình |
| **System tray** | Chạy ngầm dưới system tray với menu: Configure / Pause / Quit |
| **Phản hồi trực quan** | Highlight vùng khi kéo cửa sổ + ghost preview vị trí sẽ snap đến |
| **Đa ngôn ngữ** | Giao diện tiếng Anh + tiếng Việt (svelte-i18n) |
| **Hướng dẫn lần đầu** | Overlay onboarding khi chạy lần đầu, hướng dẫn cách tạo vùng và snap |
| **Thông báo lỗi** | Toast notification khi có lỗi từ backend (hỏng file config, lỗi save...) |
| **Tự động cập nhật** | Tauri updater — tự động tải bản mới từ GitHub Releases |

---

## Cài đặt

Grid Screen hỗ trợ 2 cách cài đặt: **cài nhanh bằng script** (khuyên dùng) hoặc **build từ source**.

---

### Cách 1: Cài nhanh bằng script (khuyên dùng)

Script tự động phát hiện hệ điều hành, cài đặt dependencies, build và cấu hình autostart.

#### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh | bash
```

Script sẽ tự động thực hiện các bước:
1. Kiểm tra và cài đặt system dependencies (libgtk-3, libwebkit2gtk, libx11...)
2. Cài đặt Rust (nếu chưa có, qua rustup)
3. Clone repository về `~/.local/share/grid-screen`
4. Build release binary (`cargo build --release`)
5. Copy binary vào `~/.local/bin/grid-screen`
6. Tạo file `.desktop` trong `~/.config/autostart/` để tự động chạy cùng hệ thống
7. Tạo desktop entry trong `~/.local/share/applications/` để hiển thị trong menu ứng dụng

Sau khi cài xong, khởi động lại (hoặc chạy `grid-screen` trực tiếp từ terminal) để bắt đầu sử dụng.

**Tùy chọn cài đặt nâng cao:**

```bash
# Cài vào thư mục tùy chỉnh
INSTALL_DIR=/opt/grid-screen bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)

# Cài bản development (có debug log)
INSTALL_MODE=dev bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)

# Chỉ cài dependencies, không build (dành cho developer)
INSTALL_MODE=deps bash <(curl -fsSL https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.sh)
```

#### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/enolalabs/grid-screen/main/scripts/install.ps1 | iex
```

Script PowerShell sẽ:
1. Cài đặt Rust qua rustup-init.exe (nếu chưa có)
2. Cài đặt Node.js qua winget (nếu chưa có)
3. Clone repository về `%LOCALAPPDATA%\grid-screen`
4. Build release binary
5. Copy vào `%LOCALAPPDATA%\Programs\grid-screen\`
6. Thêm vào registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` để autostart

---

### Cách 2: Build từ source

Dành cho developer muốn tùy chỉnh hoặc đóng góp code.

#### Yêu cầu hệ thống

| Thành phần | Phiên bản |
|------------|-----------|
| Rust | stable (1.75+) |
| Node.js | 20+ |
| npm | 9+ |
| Git | 2.0+ |

**Linux — cài thư viện hệ thống:**

```bash
# Ubuntu / Debian
sudo apt update
sudo apt install -y \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libx11-dev libxrandr-dev libxinerama-dev \
  libappindicator3-dev librsvg2-dev libdbus-1-dev \
  pkg-config libssl-dev

# Fedora
sudo dnf install -y \
  gtk3-devel webkit2gtk4.1-devel \
  libX11-devel libXrandr-devel libXinerama-devel \
  libappindicator-gtk3-devel librsvg2-devel dbus-devel \
  pkg-config openssl-devel

# Arch
sudo pacman -S --needed \
  gtk3 webkit2gtk-4.1 \
  libx11 libxrandr libxinerama \
  libappindicator-gtk3 librsvg dbus \
  pkg-config openssl
```

**Windows:** Không cần thư viện bổ sung. Đảm bảo đã cài [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) với component "Desktop development with C++".

#### Clone & build

```bash
git clone https://github.com/enolalabs/grid-screen.git
cd grid-screen

# Cài frontend dependencies
npm install

# Build
cargo tauri build
```

Binary sẽ nằm ở:
- **Linux:** `src-tauri/target/release/grid-screen`
- **Windows:** `src-tauri/target/release/grid-screen.exe`

#### Chạy development

```bash
cargo tauri dev      # Hot-reload frontend + backend
```

#### Build release

```bash
cargo tauri build --release
```

#### Cài đặt thủ công binary đã build

```bash
# Linux
sudo cp src-tauri/target/release/grid-screen /usr/local/bin/
mkdir -p ~/.config/autostart
cp assets/grid-screen.desktop ~/.config/autostart/

# Windows
# Copy grid-screen.exe vào thư mục bất kỳ, tạo shortcut trong Startup folder
# (Win+R → shell:startup → paste shortcut)
```

---

### Gỡ cài đặt

```bash
# Linux — nếu cài bằng script
~/.local/share/grid-screen/scripts/uninstall.sh

# Linux — nếu cài thủ công
rm /usr/local/bin/grid-screen
rm ~/.config/autostart/grid-screen.desktop
rm -rf ~/.config/grid-screen

# Windows — nếu cài bằng script
# Chạy "Uninstall Grid Screen" từ Start Menu

# Windows — nếu cài thủ công
# Xóa binary + shortcut trong Startup folder + registry key
reg delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v grid-screen
rmdir /s %LOCALAPPDATA%\grid-screen
```

---

## Sử dụng

### Lần đầu khởi động

Sau khi cài đặt, Grid Screen chạy ngầm dưới system tray.

1. **Click chuột phải** vào icon Grid Screen trên system tray
2. Chọn **Configure** để mở cửa sổ cấu hình
3. Một overlay **hướng dẫn** sẽ hiện ra — nhấn **Got it** để bắt đầu

### System tray menu

| Mục | Chức năng |
|-----|-----------|
| **Configure** | Mở cửa sổ cấu hình (Editor, Layouts, Settings) |
| **Pause / Resume** | Tạm dừng / bật lại chức năng snap cửa sổ |
| **View Logs** | Mở file log để debug |
| **Quit** | Thoát hoàn toàn Grid Screen |

### Tạo layout đầu tiên

1. Mở tab **Editor** trong cửa sổ cấu hình
2. Bạn sẽ thấy danh sách các màn hình đang kết nối, mỗi màn hình hiển thị dưới dạng canvas thu nhỏ
3. **Click vào canvas** của màn hình để tạo zone mới (zone mặc định chiếm 30% diện tích)
4. **Thao tác với zone:**
   - **Double-click** vào zone → đổi tên
   - **Right-click** vào zone → hiện dialog xác nhận xóa
   - **Arrow keys** → di chuyển zone (Shift để di chuyển chậm hơn)
   - **Ctrl + Arrow keys** → thay đổi kích thước zone
   - **Delete** → xóa zone đang focus
5. Zone tự động **snap vào lưới 12 cột** khi tạo và di chuyển
6. Nhấn **Apply Live** để áp dụng layout ngay lập tức — bắt đầu kéo cửa sổ vào zone
7. Nhấn **Save** để lưu layout với tên, có thể dùng lại sau

### Kéo thả cửa sổ

Sau khi layout được áp dụng:

1. **Kéo cửa sổ bất kỳ** (File Explorer, Terminal, Browser...)
2. Khi kéo qua một zone, zone đó sẽ **highlight** lên (viền tím 2px + nền mờ 20%)
3. Một **ghost preview** hiển thị vị trí cửa sổ sẽ snap đến
4. **Thả chuột** khi cursor nằm trong zone → cửa sổ tự động snap vào vị trí
5. Nếu thả chuột ngoài tất cả zone → cửa sổ giữ nguyên vị trí, không thay đổi

### Quản lý layout

- Tab **Layouts**: xem danh sách layout đã lưu
  - **Set Default** — đặt layout làm mặc định (tự động áp dụng khi khởi động)
  - **Delete** — xóa layout
- Tab **Settings**:
  - **Auto-start** — tự động chạy cùng hệ thống
  - **Default gap** — khoảng cách mặc định giữa các zone (0–100px)
  - **Default margin** — lề mặc định từ cạnh màn hình (0–100px)
  - **Accent color** — màu highlight zone và viền
  - **Language** — English / Tiếng Việt

### Phím tắt

| Phím | Chế độ | Tác dụng |
|------|--------|----------|
| **Arrow keys** | Editor — zone đang focus | Di chuyển zone |
| **Shift + Arrow** | Editor — zone đang focus | Di chuyển chậm (1% màn hình) |
| **Ctrl + Arrow** | Editor — zone đang focus | Thay đổi kích thước zone |
| **Delete** | Editor — zone đang focus | Xóa zone |

---

## Phát triển

### Cấu trúc thư mục

```
grid-screen/
├── src-tauri/                    # Rust backend (Tauri 2.x)
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri config (CSP, tray, bundler, updater)
│   ├── capabilities/
│   │   └── gridscreen.json       # Deny-by-default IPC permissions
│   ├── build.rs                  # Tauri build script
│   ├── icons/                    # Tray & app icons
│   ├── tests/                    # Integration tests
│   │   ├── config_store_tests.rs
│   │   ├── monitor_manager_tests.rs
│   │   ├── layout_manager_tests.rs
│   │   ├── zone_overlay_tests.rs
│   │   ├── drag_detector_tests.rs
│   │   └── security_smoke.rs
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri setup, IPC commands, thread wiring
│       ├── types.rs              # Shared data types (Monitor, Zone, Layout...)
│       ├── app_state.rs          # AppState with ArcSwap/RwLock/Mutex
│       ├── config_store.rs       # JSON config read/write + validation + backup rotation
│       ├── monitor_manager.rs    # Event-driven monitor detection + 30s safety-net polling
│       ├── layout_manager.rs     # Fractional coordinate math, zone ops
│       ├── drag_detector.rs      # Event-driven drag processor (blocking mpsc)
│       ├── zone_overlay.rs       # Transparent overlay rendering (tiny-skia)
│       ├── user_notifier.rs      # Backend → frontend error bridging
│       ├── perf.rs               # Frame counter & FPS stats
│       └── platform/
│           ├── mod.rs            # PlatformApi trait + conditional exports
│           ├── mock.rs           # Mock implementation (cho testing)
│           ├── linux.rs          # X11 implementation (x11rb, RandR, Xinerama)
│           └── windows.rs        # Windows stub (TODO: EnumDisplayMonitors)
├── src/                          # Frontend (Svelte 5 + TypeScript)
│   ├── main.ts                   # App mount point
│   ├── App.svelte                # App shell (tabs, onboarding, toast)
│   ├── vite-env.d.ts             # Vite type declarations
│   ├── lib/
│   │   ├── types.ts              # TypeScript types mirroring Rust types
│   │   ├── ipc.ts                # Tauri invoke wrappers
│   │   ├── stores.ts             # Svelte stores (currentState, layouts...)
│   │   ├── notifications.ts      # Toast notification store
│   │   ├── i18n.ts               # i18n initialization
│   │   └── i18n/
│   │       ├── en.json           # English strings
│   │       └── vi.json           # Vietnamese strings
│   └── routes/
│       ├── LayoutEditor.svelte   # WYSIWYG zone editor (ARIA + keyboard a11y)
│       ├── LayoutManager.svelte  # Saved layouts list
│       ├── Settings.svelte       # Settings form
│       └── __tests__/
│           └── LayoutEditor.test.ts  # Component + keyboard nav tests
├── benches/
│   └── overlay_bench.rs          # Criterion benchmark (zone hit-test 64 zones)
├── .github/workflows/
│   └── ci.yml                    # CI matrix: ubuntu-latest + windows-latest
├── vitest.config.ts              # Vitest config (jsdom environment)
├── package.json                  # Node deps
└── svelte.config.js              # Svelte compiler config
```

### Chạy development

```bash
# Cài dependencies
npm install

# Chạy Tauri dev mode (hot-reload frontend + backend)
cargo tauri dev
```

### Các lệnh hữu ích

```bash
# Backend
cargo fmt --check          # Kiểm tra format Rust
cargo clippy -- -D warnings # Lint Rust
cargo test                  # Chạy toàn bộ test Rust
cargo bench                 # Chạy benchmark
cargo build --release       # Build production

# Frontend
npx vitest run              # Chạy test component
npm run build               # Build Svelte production

# Security
cargo audit                 # Kiểm tra CVE trong dependencies
cargo deny check            # Kiểm tra license + banned deps
```

---

## Kiến trúc

### Mô hình luồng (Threading)

Grid Screen chạy theo mô hình 4-thread:

| Thread | Vai trò | Cơ chế đồng bộ |
|--------|---------|----------------|
| **Main** | Tauri runtime + system tray + IPC | — |
| **Platform event loop** | X11/Win32 event polling (window move, display change) | `mpsc::channel` → DragDetector |
| **Drag processor** | Nhận event, hit-test zone, gửi snap, điều khiển overlay | Blocking `mpsc::recv()` |
| **Overlay renderer** | Vẽ zone highlight + ghost rect qua tiny-skia | Gọi từ DragDetector callbacks |

### Truy cập state

| Dữ liệu | Cơ chế | Lý do |
|---------|--------|------|
| `monitors` | `ArcSwap<Vec<Monitor>>` | Lock-free read — hotpath |
| `active_layouts` | `ArcSwap<Vec<Layout>>` | Lock-free read — hotpath |
| `drag_state` | `Mutex<Option<DragState>>` | Chỉ truy cập từ DragDetector thread |
| `app_config` | `RwLock<AppConfig>` | Nhiều reader, ít writer (settings page) |

### Platform API

Backend sử dụng trait `PlatformApi` để trừu tượng hóa các API của hệ điều hành:

```rust
pub trait PlatformApi: Send + Sync {
    fn enumerate_monitors(&self) -> Vec<Monitor>;
    fn enumerate_windows(&self) -> Vec<Window>;
    fn move_window(&self, handle: WindowHandle, rect: Rect);
    fn get_cursor_pos(&self) -> (i32, i32);
    fn is_mouse_button_down(&self) -> bool;
    fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent>;
    fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent>;
    fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String>;
    fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32);
    fn destroy_overlay_window(&self, handle: OverlayHandle);
    fn set_autostart(&self, enabled: bool) -> Result<(), String>;
}
```

| Implementation | Nền tảng | Thư viện |
|----------------|----------|----------|
| `LinuxPlatformApi` | X11 | `x11rb` (RandR, Xinerama, Shape, ConfigureWindow) |
| `WindowsPlatformApi` | Windows | `windows` crate (EnumDisplayMonitors, SetWindowPos...) |
| `MockPlatformApi` | Testing | Không phụ thuộc OS, dùng trong unit/integration test |

### Luồng xử lý drag

```
Người dùng kéo cửa sổ
  → Platform API bắt WindowMoveEvent
    → DragDetector nhận event qua blocking mpsc::recv()
      → DragStart: lấy vị trí cursor, tìm monitor, hiện overlay
      → DragMove: hit-test zone (O(n), max 64 zones), cập nhật highlight + ghost
      → DragEnd: nếu cursor trong zone → gửi SnapEvent
        → Snap consumer thread gọi platform_api.move_window()
```

### Tọa độ fractional

Tất cả zone được lưu dưới dạng tọa độ phân số (0.0–1.0) so với kích thước màn hình:

```
zone.x = 0.0, zone.width = 0.5  → chiếm nửa trái màn hình (mọi độ phân giải)
zone.y = 0.5, zone.height = 0.5 → chiếm nửa dưới màn hình
```

Chuyển đổi sang pixel khi cần sử dụng tại runtime:
```
pixel_x = monitor.x + zone.x * monitor.width + margin + gap/2
pixel_w = zone.width * monitor.width - 2*margin - gap
```

---

## Cấu hình

### File config

Grid Screen lưu cấu hình tại `$XDG_CONFIG_HOME/grid-screen/layouts.json` (Linux) hoặc `%APPDATA%/grid-screen/layouts.json` (Windows).

```json
{
  "schema_version": 1,
  "layouts": [
    {
      "id": "uuid",
      "name": "My Layout",
      "arrangement_id": "monitor_hash",
      "monitor_id": "uuid",
      "zones": [
        {
          "id": "uuid",
          "name": "Left Half",
          "x": 0.0, "y": 0.0,
          "width": 0.5, "height": 1.0,
          "gap": 4, "margin": 8
        }
      ]
    }
  ],
  "settings": {
    "auto_start": false,
    "default_gap": 4,
    "default_margin": 8,
    "accent_color": "#7C3AED",
    "language": "en",
    "first_run_completed": false
  }
}
```

### Validation

- Schema version check
- Tên zone/layout: 1–64 ký tự
- Tối đa 64 zone / màn hình
- Tọa độ trong khoảng [0.0, 1.0]
- Zone không được chồng lấn
- HTML escape tên zone trước khi lưu
- Backup rotation: giữ 5 file `.bak.N` gần nhất
- Atomic write: ghi ra file `.tmp` → verify → rename

### Logging

Log lưu tại `$XDG_CONFIG_HOME/grid-screen/grid-screen.log`:
- Rotation hàng ngày (daily)
- Tối đa 3 file, mỗi file ≤ 1MB
- Panic hook tự động ghi backtrace vào log

---

## Build & phân phối

### Build local

```bash
npm install
cargo tauri build
```

### Build CI

GitHub Actions (`ubuntu-latest` + `windows-latest` matrix):
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo audit`
5. `cargo deny check`
6. `npm ci && npx vitest run`
7. `cargo build --release`

### Định dạng phân phối

| Nền tảng | Định dạng |
|----------|-----------|
| Linux | `.deb` (Debian/Ubuntu), `.AppImage` (universal) |
| Windows | `.msi` (system-wide), NSIS `.exe` (per-user) |

### Auto-update

Cấu hình updater sử dụng Tauri updater plugin, kiểm tra bản mới từ GitHub Releases:

```
GET https://github.com/enolalabs/grid-screen/releases/latest/download/latest.json
```

### Bảo mật

- **Capabilities**: deny-by-default — chỉ cấp `core:default`, `tray:default`, và các quyền window cơ bản. Không `shell:`, `http:`, `fs:`.
- **CSP**: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self' ipc: https://ipc.localhost; img-src 'self' data:;`
- **File permissions**: config file được set `0o600` (chỉ owner đọc/ghi) trên Linux
- **HTML escaping**: tên zone được escape trước khi lưu (ngăn stored XSS)
- **Smoke tests**: `security_smoke.rs` tự động kiểm tra capabilities và CSP

---

## Kiểm thử

### Unit & Integration tests (Rust)

```bash
cargo test                                    # Tất cả test
cargo test config_store                       # Test ConfigStore (6 tests)
cargo test monitor_manager                    # Test MonitorManager (2 tests)
cargo test layout_manager                     # Test LayoutManager (3 tests)
cargo test drag_detector                      # Test DragDetector (2 tests)
cargo test zone_overlay                       # Test ZoneOverlay (3 tests)
cargo test security_smoke                     # Test security (2 tests)
```

### Component tests (Svelte)

```bash
npx vitest run                                # LayoutEditor component + keyboard a11y
```

### Benchmark

```bash
cargo bench                                   # Zone hit-test 64 zones @ 4K
```

Budget: `hit_test_64_zones` < 1ms (đảm bảo fit trong 16ms frame budget kể cả ở max zones).

---

## Đóng góp

1. Fork repository
2. Tạo branch: `git checkout -b feature/ten-tinh-nang`
3. Code + test: `cargo test && npx vitest run`
4. Format: `cargo fmt && cargo clippy -- -D warnings`
5. Commit với message rõ ràng
6. Push và tạo Pull Request

**Quy ước commit:** [Conventional Commits](https://www.conventionalcommits.org/)
- `feat:` — tính năng mới
- `fix:` — sửa lỗi
- `test:` — thêm/sửa test
- `chore:` — config, build, CI
- `docs:` — documentation

### Tài liệu tham khảo

- [Design spec](docs/superpowers/specs/2026-07-09-grid-screen-design.md)
- [Implementation plan](docs/superpowers/plans/2026-07-09-grid-screen-implementation.md)
- [Tauri 2.x docs](https://v2.tauri.app/)
- [Svelte 5 docs](https://svelte.dev/docs)
- [x11rb docs](https://docs.rs/x11rb)

### Roadmap

| Giai đoạn | Nội dung |
|-----------|----------|
| **v0.1** (hiện tại) | X11 backend, Windows stub, Svelte 5 config UI, tray |
| **v0.2** | Windows backend hoàn chỉnh (EnumDisplayMonitors, SetWindowPos, Event Hook) |
| **v0.3** | Wayland hỗ trợ (qua XWayland), pixel format conversion cho overlay |
| **v1.0** | macOS support, keyboard shortcuts, auto-layout gợi ý |

---

## License

MIT
