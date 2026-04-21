#[path = "discord_presence/feed.rs"]
mod feed;
#[path = "discord_presence/ipc.rs"]
mod ipc;
#[path = "discord_presence/messages.rs"]
mod messages;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use reqwest::blocking::Client;

use crate::{
    backend_locale::BackendLocale,
    http_client::build_blocking_client,
    models::{ClientConfig, DiscordPresenceSnapshot},
    runtime_utils::{now_iso_string, sleep_with_stop, wait_for_worker_exit},
};

use feed::{fetch_public_activity_feed_blocking, select_dc_source_activity};
use ipc::{apply_discord_presence, clear_discord_presence, map_activity_to_presence};
use messages::{
    discord_config_app_id_missing, discord_config_base_url_missing,
    discord_config_source_id_missing, discord_worker_stopping,
};

const SYNC_INTERVAL: Duration = Duration::from_secs(15);
const MAX_ERROR_BACKOFF_MS: u64 = 60_000;
const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);

#[derive(Default)]
struct DiscordPresenceInner {
    running: bool,
    active_run_id: Option<u64>,
    connected: bool,
    last_sync_at: Option<String>,
    last_error: Option<String>,
    current_summary: Option<String>,
    stop_flag: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

pub struct DiscordPresenceRuntime {
    inner: Arc<Mutex<DiscordPresenceInner>>,
    sequence: AtomicU64,
}

impl DiscordPresenceRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DiscordPresenceInner::default())),
            sequence: AtomicU64::new(1),
        }
    }

    pub fn snapshot(&self) -> DiscordPresenceSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        snapshot_from_inner(&inner)
    }

    pub fn start(
        &self,
        config: ClientConfig,
        locale: BackendLocale,
    ) -> Result<DiscordPresenceSnapshot, String> {
        validate_discord_presence_config(&config, locale)?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let run_id = self.sequence.fetch_add(1, Ordering::SeqCst);
        let previous_worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.running {
                return Ok(snapshot_from_inner(&inner));
            }

            if let Some(old_flag) = inner.stop_flag.take() {
                old_flag.store(true, Ordering::SeqCst);
            }

            inner.worker.take()
        };

        if let Some(handle) = previous_worker {
            if let Err(handle) =
                wait_for_worker_exit(handle, WORKER_JOIN_WAIT_TIMEOUT, WORKER_JOIN_POLL_STEP)
            {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
                return Err(discord_worker_stopping(locale));
            }
        }

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.connected = false;
            inner.last_error = None;
            inner.stop_flag = Some(stop_flag.clone());
        }

        let state = self.inner.clone();
        let worker = thread::spawn(move || {
            run_discord_presence_loop(state, config, stop_flag, run_id, locale);
        });

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = Some(worker);
        }

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> DiscordPresenceSnapshot {
        let worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
            inner.connected = false;
            inner.worker.take()
        };

        if let Some(handle) = worker {
            if let Err(handle) =
                wait_for_worker_exit(handle, WORKER_JOIN_WAIT_TIMEOUT, WORKER_JOIN_POLL_STEP)
            {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
            } else {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = None;
            }
        }

        self.snapshot()
    }
}

impl Drop for DiscordPresenceRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PresenceUpdate {
    Set(String),
    Cleared,
}

fn snapshot_from_inner(inner: &DiscordPresenceInner) -> DiscordPresenceSnapshot {
    DiscordPresenceSnapshot {
        running: inner.running,
        connected: inner.connected,
        last_sync_at: inner.last_sync_at.clone(),
        last_error: inner.last_error.clone(),
        current_summary: inner.current_summary.clone(),
    }
}

fn run_discord_presence_loop(
    state: Arc<Mutex<DiscordPresenceInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    run_id: u64,
    locale: BackendLocale,
) {
    let http_client = match build_http_client(config.use_system_proxy, locale) {
        Ok(client) => client,
        Err(error) => {
            set_discord_error(&state, Some(error), false, run_id);
            mark_stopped(&state, run_id);
            return;
        }
    };

    let target_dc_source = config.discord_source_id.trim().to_string();
    let mut discord_client = None;
    let mut consecutive_errors = 0u32;

    while !stop_flag.load(Ordering::SeqCst) {
        match fetch_public_activity_feed_blocking(&http_client, &config.base_url, locale) {
            Ok(feed) => {
                let target_activity =
                    select_dc_source_activity(&feed.active_statuses, &target_dc_source);

                let result = if let Some(activity) = target_activity {
                    let payload = map_activity_to_presence(activity);
                    apply_discord_presence(
                        &mut discord_client,
                        &config.discord_application_id,
                        &payload,
                        locale,
                    )
                    .map(|()| PresenceUpdate::Set(payload.summary))
                } else {
                    clear_discord_presence(
                        &mut discord_client,
                        &config.discord_application_id,
                        locale,
                    )
                    .map(|()| PresenceUpdate::Cleared)
                };

                match result {
                    Ok(PresenceUpdate::Set(summary)) => {
                        update_presence_snapshot(&state, true, None, Some(summary), run_id);
                        consecutive_errors = 0;
                        sleep_with_stop(SYNC_INTERVAL, &stop_flag);
                    }
                    Ok(PresenceUpdate::Cleared) => {
                        update_presence_snapshot(&state, true, None, None, run_id);
                        consecutive_errors = 0;
                        sleep_with_stop(SYNC_INTERVAL, &stop_flag);
                    }
                    Err(error) => {
                        discord_client = None;
                        consecutive_errors = consecutive_errors.saturating_add(1);
                        set_discord_error(&state, Some(error), false, run_id);
                        sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
                    }
                }
            }
            Err(error) => {
                consecutive_errors = consecutive_errors.saturating_add(1);
                set_discord_error_preserving_connection(&state, Some(error), run_id);
                sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
            }
        }
    }

    let _ = clear_discord_presence(&mut discord_client, &config.discord_application_id, locale);
    mark_stopped(&state, run_id);
}

fn validate_discord_presence_config(
    config: &ClientConfig,
    locale: BackendLocale,
) -> Result<(), String> {
    if config.base_url.trim().is_empty() {
        return Err(discord_config_base_url_missing(locale));
    }
    if config.discord_application_id.trim().is_empty() {
        return Err(discord_config_app_id_missing(locale));
    }
    if config.discord_source_id.trim().is_empty() {
        return Err(discord_config_source_id_missing(locale));
    }
    Ok(())
}

pub fn config_is_ready(config: &ClientConfig) -> bool {
    validate_discord_presence_config(config, BackendLocale::ZhCn).is_ok()
}

fn update_presence_snapshot(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    connected: bool,
    last_error: Option<String>,
    current_summary: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = last_error;
    inner.last_sync_at = Some(now_iso_string());
    inner.current_summary = current_summary;
}

fn set_discord_error(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    error: Option<String>,
    connected: bool,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = error;
}

fn set_discord_error_preserving_connection(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    error: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.last_error = error;
}

fn mark_stopped(state: &Arc<Mutex<DiscordPresenceInner>>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.running = false;
    inner.connected = false;
    inner.active_run_id = None;
    inner.stop_flag = None;
}

fn build_http_client(use_system_proxy: bool, locale: BackendLocale) -> Result<Client, String> {
    build_blocking_client(
        "waken-wa-tauri-discord-presence/0.1.0",
        Some(Duration::from_secs(15)),
        use_system_proxy,
        locale,
    )
}

fn error_backoff(consecutive_errors: u32) -> Duration {
    let multiplier = 2u64.saturating_pow(consecutive_errors.saturating_sub(1));
    Duration::from_millis(
        (SYNC_INTERVAL.as_millis() as u64)
            .saturating_mul(multiplier.max(1))
            .min(MAX_ERROR_BACKOFF_MS),
    )
}

#[cfg(test)]
mod tests {
    use super::{map_activity_to_presence, select_dc_source_activity};
    use crate::discord_presence::feed::{item_dc_source, PublicActivityItem};
    use crate::models::AppStatePayload;
    use serde_json::json;

    #[test]
    fn selects_only_matching_dc_source_activity() {
        let items = vec![
            PublicActivityItem {
                metadata: Some(json!({ "dc_source": "wwd-other" })),
                status_text: Some("Writing".into()),
                ..Default::default()
            },
            PublicActivityItem {
                metadata: Some(json!({ "dc_source": "wwd-self" })),
                status_text: Some("Reviewing".into()),
                ..Default::default()
            },
        ];

        let selected = select_dc_source_activity(&items, "wwd-self").unwrap();
        assert_eq!(selected.status_text.as_deref(), Some("Reviewing"));
    }

    #[test]
    fn ignores_other_sources() {
        let items = vec![PublicActivityItem {
            metadata: Some(json!({ "dc_source": "wwd-other" })),
            status_text: Some("Streaming".into()),
            ..Default::default()
        }];

        assert!(select_dc_source_activity(&items, "wwd-self").is_none());
    }

    #[test]
    fn extracts_dc_source_from_metadata() {
        let item = PublicActivityItem {
            metadata: Some(json!({ "dc_source": "wwd-self" })),
            ..Default::default()
        };

        assert_eq!(item_dc_source(&item), Some("wwd-self"));
    }

    #[test]
    fn maps_status_text_then_device_to_presence() {
        let payload = map_activity_to_presence(&PublicActivityItem {
            device: Some("Desk PC".into()),
            status_text: Some("Fixing Discord sync".into()),
            started_at: Some("2026-04-16T10:00:00Z".into()),
            ..Default::default()
        });

        assert_eq!(payload.details, "Fixing Discord sync");
        assert_eq!(payload.state.as_deref(), Some("Desk PC"));
        assert_eq!(payload.summary, "Fixing Discord sync · Desk PC");
        assert_eq!(payload.started_at_millis, Some(1_776_333_600_000));
    }

    #[test]
    fn falls_back_to_process_title_and_name() {
        let payload = map_activity_to_presence(&PublicActivityItem {
            device: Some("Desk PC".into()),
            process_name: Some("Code.exe".into()),
            process_title: Some("src/App.vue".into()),
            ..Default::default()
        });

        assert_eq!(payload.details, "src/App.vue | Code.exe");
    }

    #[test]
    fn legacy_state_defaults_new_discord_fields() {
        let payload: AppStatePayload = serde_json::from_str(
            r#"{
              "config": {
                "baseUrl": "https://example.com",
                "apiToken": "token",
                "generatedHashKey": "wwd-1"
              },
              "recentPresets": [],
              "onboardingDismissed": false
            }"#,
        )
        .unwrap();

        assert!(!payload.config.discord_enabled);
        assert_eq!(payload.config.discord_application_id, "");
        assert!(payload.config.discord_source_id.is_empty());
    }
}
