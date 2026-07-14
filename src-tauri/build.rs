fn main() {
    // Compile Wayland helper C code (optional — only on Linux with wayland-client)
    let wayland_available = pkg_config::probe_library("wayland-client").is_ok();
    if wayland_available {
        cc::Build::new()
            .file("wayland-protocol/wayland-helper.c")
            .file("wayland-protocol/ext-foreign-toplevel-list-v1.c")
            .include("wayland-protocol")
            .compile("wayland-helper");
    }

    tauri_build::build()
}
