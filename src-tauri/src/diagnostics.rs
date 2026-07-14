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
