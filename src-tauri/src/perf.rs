use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);
static FRAME_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub fn record_frame() {
    FRAME_START.get_or_init(Instant::now);
    FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn current_fps() -> f64 {
    let elapsed = FRAME_START.get().map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.001);
    let count = FRAME_COUNT.load(Ordering::Relaxed) as f64;
    count / elapsed.max(0.001)
}
