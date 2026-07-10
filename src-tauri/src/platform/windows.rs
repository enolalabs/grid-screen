#[cfg(target_os = "windows")]
mod windows_impl {
    use std::sync::mpsc;
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Registry::*;
    use windows::Win32::UI::Input::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    use super::PlatformApi;
    use crate::types::*;

    static OVERLAY_CLASS_ATOM: OnceLock<u16> = OnceLock::new();

    fn register_overlay_class() -> Result<(), String> {
        OVERLAY_CLASS_ATOM.get_or_init(|| {
            let class_name = windows::core::w!("GridScreenOverlay");
            let instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(overlay_wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: HICON::default(),
                hCursor: HCURSOR::default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: windows::core::w!(""),
                lpszClassName: class_name,
            };
            unsafe { RegisterClassW(&wc) }
        });
        Ok(())
    }

    pub struct WindowsPlatformApi;

    impl WindowsPlatformApi {
        pub fn new() -> Result<Self, String> {
            Ok(Self)
        }
    }

    impl PlatformApi for WindowsPlatformApi {
        fn enumerate_monitors(&self) -> Vec<Monitor> {
            let mut monitors = Vec::new();
            unsafe {
                EnumDisplayMonitors(
                    HDC::default(),
                    None,
                    Some(monitor_enum_proc),
                    LPARAM(&mut monitors as *mut _ as isize),
                );
            }
            monitors
        }

        fn enumerate_windows(&self) -> Vec<Window> {
            let mut windows = Vec::new();
            unsafe {
                EnumWindows(
                    Some(window_enum_proc),
                    LPARAM(&mut windows as *mut _ as isize),
                );
            }
            windows
        }

        fn move_window(&self, handle: WindowHandle, rect: Rect) {
            unsafe {
                let hwnd = HWND(handle.0 as *mut _);
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    rect.x,
                    rect.y,
                    rect.width as i32,
                    rect.height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }

        fn get_cursor_pos(&self) -> (i32, i32) {
            let mut pt = POINT::default();
            unsafe {
                GetCursorPos(&mut pt).unwrap_or_default();
            }
            (pt.x, pt.y)
        }

        fn is_mouse_button_down(&self) -> bool {
            unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON)) as u16 & 0x8000 != 0 }
        }

        fn subscribe_window_move_events(&self) -> mpsc::Receiver<WindowMoveEvent> {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let mut prev_windows: Vec<(u64, Rect)> = Vec::new();
                let mut drag_handle: Option<u64> = None;
                loop {
                    thread::sleep(Duration::from_millis(60));

                    let mouse_down =
                        unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON)) as u16 & 0x8000 != 0 };

                    let mut current = Vec::new();
                    unsafe {
                        EnumWindows(
                            Some(enum_for_tracking),
                            LPARAM(&mut current as *mut _ as isize),
                        );
                    }

                    if mouse_down {
                        for (hwnd_val, rect) in &current {
                            let prev_rect = prev_windows
                                .iter()
                                .find(|(h, _)| h == hwnd_val)
                                .map(|(_, r)| *r);

                            if let Some(pr) = prev_rect {
                                if pr.x != rect.x
                                    || pr.y != rect.y
                                    || pr.width != rect.width
                                    || pr.height != rect.height
                                {
                                    if drag_handle == Some(*hwnd_val) {
                                        let _ = tx.send(WindowMoveEvent::DragMove {
                                            handle: WindowHandle(*hwnd_val),
                                            rect: *rect,
                                        });
                                    } else {
                                        drag_handle = Some(*hwnd_val);
                                        let _ = tx.send(WindowMoveEvent::DragStart {
                                            handle: WindowHandle(*hwnd_val),
                                            rect: *rect,
                                        });
                                    }
                                }
                            }
                        }
                    } else if let Some(dh) = drag_handle.take() {
                        if let Some(rect) = current.iter().find(|(h, _)| *h == dh).map(|(_, r)| *r)
                        {
                            let _ = tx.send(WindowMoveEvent::DragEnd {
                                handle: WindowHandle(dh),
                                rect,
                            });
                        }
                    }

                    prev_windows = current;
                }
            });
            rx
        }

        fn subscribe_display_change_events(&self) -> mpsc::Receiver<DisplayChangeEvent> {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let mut last_count = 0u32;
                loop {
                    thread::sleep(Duration::from_secs(3));
                    let count = unsafe { GetSystemMetrics(SM_CMONITORS) as u32 };
                    if count != last_count && last_count != 0 {
                        if count > last_count {
                            let _ = tx.send(DisplayChangeEvent::Connected);
                        } else {
                            let _ = tx.send(DisplayChangeEvent::Disconnected);
                        }
                    }
                    last_count = count;
                }
            });
            rx
        }

        fn create_overlay_window(&self, monitor_id: MonitorId) -> Result<OverlayHandle, String> {
            register_overlay_class()?;

            let monitors = self.enumerate_monitors();
            let mon = monitors.iter().find(|m| m.id == monitor_id).cloned();

            let (mx, my, mw, mh) = match mon {
                Some(ref m) => (m.x, m.y, m.width as i32, m.height as i32),
                None => return Err("Monitor not found".into()),
            };

            let class_name = windows::core::w!("GridScreenOverlay");
            unsafe {
                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED
                        | WS_EX_TRANSPARENT
                        | WS_EX_TOOLWINDOW
                        | WS_EX_NOACTIVATE
                        | WS_EX_TOPMOST,
                    class_name,
                    windows::core::w!(""),
                    WS_POPUP,
                    mx,
                    my,
                    mw,
                    mh,
                    HWND::default(),
                    None,
                    HINSTANCE::default(),
                    None,
                );
                if hwnd.0.is_null() {
                    return Err("Failed to create overlay window".into());
                }
                SetLayeredWindowAttributes(hwnd, COLORREF::default(), 200, LWA_ALPHA);
                ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                Ok(OverlayHandle(hwnd.0 as u64))
            }
        }

        fn overlay_present(&self, handle: &OverlayHandle, pixels: &[u8], w: u32, h: u32) {
            unsafe {
                let hwnd = HWND(handle.0 as *mut _);

                let mut bmi = BITMAPINFO::default();
                bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bmi.bmiHeader.biWidth = w as i32;
                bmi.bmiHeader.biHeight = -(h as i32);
                bmi.bmiHeader.biPlanes = 1;
                bmi.bmiHeader.biBitCount = 32;
                bmi.bmiHeader.biCompression = BI_RGB;

                let hdc = GetDC(hwnd);
                if hdc.is_invalid() {
                    return;
                }

                let mut bit_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                let bitmap = CreateDIBSection(
                    hdc,
                    &bmi,
                    DIB_RGB_COLORS,
                    &mut bit_ptr,
                    HINSTANCE::default(),
                    0,
                );

                if !bitmap.is_invalid() && !bit_ptr.is_null() {
                    let dst = bit_ptr as *mut u8;
                    std::ptr::copy_nonoverlapping(pixels.as_ptr(), dst, (w * h * 4) as usize);

                    let mem_dc = CreateCompatibleDC(hdc);
                    if !mem_dc.is_invalid() {
                        let old = SelectObject(mem_dc, bitmap);
                        let blend = BLENDFUNCTION {
                            BlendOp: AC_SRC_OVER as u8,
                            BlendFlags: 0,
                            SourceConstantAlpha: 255,
                            AlphaFormat: AC_SRC_ALPHA as u8,
                        };

                        let pt_dst = POINT { x: 0, y: 0 };
                        let sz = windows::Win32::Foundation::SIZE {
                            cx: w as i32,
                            cy: h as i32,
                        };

                        UpdateLayeredWindow(
                            hwnd,
                            hdc,
                            Some(&pt_dst),
                            Some(&sz),
                            mem_dc,
                            Some(&POINT::default()),
                            COLORREF::default(),
                            Some(&blend),
                            ULW_ALPHA,
                        );
                        SelectObject(mem_dc, old);
                        DeleteDC(mem_dc);
                    }
                    DeleteObject(bitmap);
                }
                ReleaseDC(hwnd, hdc);
            }
        }

        fn destroy_overlay_window(&self, handle: OverlayHandle) {
            unsafe {
                DestroyWindow(HWND(handle.0 as *mut _));
            }
        }

        fn set_autostart(&self, enabled: bool) -> Result<(), String> {
            unsafe {
                let key_path =
                    windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
                let mut hkey = HKEY::default();
                let result =
                    RegOpenKeyExW(HKEY_CURRENT_USER, key_path, 0, KEY_SET_VALUE, &mut hkey);
                if result != ERROR_SUCCESS {
                    return Err("Cannot open registry key".into());
                }

                if enabled {
                    let exe = std::env::current_exe().map_err(|e| format!("{}", e))?;
                    let exe_str = exe.to_string_lossy();
                    let value = windows::core::w!("grid-screen");
                    let data = encode_wide(&exe_str);
                    let data_bytes: &[u8] =
                        std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2);
                    RegSetValueExW(hkey, value, 0, REG_SZ, Some(data_bytes))
                        .map_err(|e| format!("Registry write failed: {}", e))?;
                } else {
                    let value = windows::core::w!("grid-screen");
                    RegDeleteValueW(hkey, value)
                        .map_err(|e| format!("Registry delete failed: {}", e))?;
                }

                RegCloseKey(hkey);
            }
            Ok(())
        }
    }

    // ── Win32 callbacks ────────────────────────────────────────

    unsafe extern "system" fn monitor_enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(
            hmonitor,
            &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
        .as_bool()
        {
            let name = String::from_utf16_lossy(&info.szDevice);
            monitors.push(Monitor {
                id: MonitorId::from_name(&name),
                name,
                x: info.monitorInfo.rcMonitor.left,
                y: info.monitorInfo.rcMonitor.top,
                width: (info.monitorInfo.rcMonitor.right - info.monitorInfo.rcMonitor.left) as u32,
                height: (info.monitorInfo.rcMonitor.bottom - info.monitorInfo.rcMonitor.top) as u32,
                dpi_scale: 1.0,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            });
        }
        BOOL::from(true)
    }

    unsafe extern "system" fn window_enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<Window>);
        if IsWindowVisible(hwnd).as_bool() {
            let mut title_buf = [0u16; 256];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let title = String::from_utf16_lossy(&title_buf[..len as usize]);
            if !title.is_empty() {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).as_bool() {
                    windows.push(Window {
                        handle: WindowHandle(hwnd.0 as u64),
                        title,
                        rect: Rect {
                            x: rect.left,
                            y: rect.top,
                            width: (rect.right - rect.left) as u32,
                            height: (rect.bottom - rect.top) as u32,
                        },
                        is_visible: true,
                    });
                }
            }
        }
        BOOL::from(true)
    }

    unsafe extern "system" fn enum_for_tracking(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(u64, Rect)>);
        if IsWindowVisible(hwnd).as_bool() {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                let w = (rect.right - rect.left) as u32;
                let h = (rect.bottom - rect.top) as u32;
                if w > 0 && h > 0 {
                    out.push((
                        hwnd.0 as u64,
                        Rect {
                            x: rect.left,
                            y: rect.top,
                            width: w,
                            height: h,
                        },
                    ));
                }
            }
        }
        BOOL::from(true)
    }

    unsafe extern "system" fn overlay_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    fn encode_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::WindowsPlatformApi;
