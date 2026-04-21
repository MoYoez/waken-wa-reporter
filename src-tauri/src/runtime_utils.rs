use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;

pub fn sleep_with_stop(duration: Duration, stop_flag: &Arc<AtomicBool>) {
    let mut remaining = duration.as_millis() as u64;
    while remaining > 0 {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let step = remaining.min(200);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}

pub fn wait_for_worker_exit(
    handle: JoinHandle<()>,
    timeout: Duration,
    poll_step: Duration,
) -> Result<(), JoinHandle<()>> {
    let deadline = Instant::now() + timeout;
    let handle = handle;

    while Instant::now() < deadline {
        if handle.is_finished() {
            let _ = handle.join();
            return Ok(());
        }

        thread::sleep(poll_step);
    }

    if handle.is_finished() {
        let _ = handle.join();
        Ok(())
    } else {
        Err(handle)
    }
}

pub fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}
