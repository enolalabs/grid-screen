use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
pub struct UserNotification {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

impl UserNotification {
    pub fn info(message: &str) -> Self {
        Self { level: "info".into(), message: message.into(), timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }
    }
    pub fn warning(message: &str) -> Self {
        Self { level: "warning".into(), message: message.into(), timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }
    }
    pub fn error(message: &str) -> Self {
        Self { level: "error".into(), message: message.into(), timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }
    }
}

pub fn notify(app_handle: &tauri::AppHandle, notification: UserNotification) {
    let _ = app_handle.emit("user-notification", notification);
}
