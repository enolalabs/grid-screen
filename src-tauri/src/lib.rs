use tauri::Manager;

mod platform;
mod types;
pub mod config_store;

#[tauri::command]
fn get_current_state() -> String {
    serde_json::json!({"status": "ok", "monitors": [], "layout": null}).to_string()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_current_state])
        .setup(|app| {
            let _window = tauri::WebviewWindowBuilder::new(
                app,
                "config-main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Grid Screen")
            .inner_size(800.0, 600.0)
            .visible(false)
            .build()?;
            tracing::info!("Grid Screen started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Grid Screen");
}
