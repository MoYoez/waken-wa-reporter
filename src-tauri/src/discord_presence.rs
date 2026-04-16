use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    http_client::build_blocking_client,
    models::{ClientConfig, DiscordPresenceSnapshot},
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

    pub fn start(&self, config: ClientConfig) -> Result<DiscordPresenceSnapshot, String> {
        validate_discord_presence_config(&config)?;

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
            if let Err(handle) = wait_for_worker_exit(handle) {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
                return Err("上一次 Discord 同步线程仍在退出，请稍后重试。".into());
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
            run_discord_presence_loop(state, config, stop_flag, run_id);
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
            if let Err(handle) = wait_for_worker_exit(handle) {
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

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PublicActivityFeed {
    #[serde(default)]
    active_statuses: Vec<PublicActivityItem>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PublicActivityItem {
    #[serde(default)]
    device: Option<String>,
    #[serde(default)]
    metadata: Option<Value>,
    #[serde(default)]
    process_name: Option<String>,
    #[serde(default)]
    process_title: Option<String>,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    started_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscordPresencePayload {
    details: String,
    state: Option<String>,
    started_at_millis: Option<i64>,
    summary: String,
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
) {
    let http_client = match build_http_client(config.use_system_proxy) {
        Ok(client) => client,
        Err(error) => {
            set_discord_error(&state, Some(error), false, run_id);
            mark_stopped(&state, run_id);
            return;
        }
    };

    let target_dc_source = config.discord_source_id.trim().to_string();
    let mut discord_client: Option<DiscordIpcClient> = None;
    let mut consecutive_errors = 0u32;

    while !stop_flag.load(Ordering::SeqCst) {
        match fetch_public_activity_feed_blocking(&http_client, &config.base_url) {
            Ok(feed) => {
                let target_activity =
                    select_dc_source_activity(&feed.active_statuses, &target_dc_source);

                let result = if let Some(activity) = target_activity {
                    let payload = map_activity_to_presence(activity);
                    apply_discord_presence(
                        &mut discord_client,
                        &config.discord_application_id,
                        &payload,
                    )
                    .map(|()| PresenceUpdate::Set(payload.summary))
                } else {
                    clear_discord_presence(&mut discord_client, &config.discord_application_id)
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

    let _ = clear_discord_presence(&mut discord_client, &config.discord_application_id);
    mark_stopped(&state, run_id);
}

enum PresenceUpdate {
    Set(String),
    Cleared,
}

fn fetch_public_activity_feed_blocking(
    client: &Client,
    base_url: &str,
) -> Result<PublicActivityFeed, String> {
    let url = format!("{}/api/activity?public=1", base_url.trim_end_matches('/'));
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .map_err(|error| format!("拉取公开活动失败：{error}"))?;

    let status = response.status().as_u16();
    let text = response.text().unwrap_or_default();
    if status >= 400 {
        return Err(if text.trim().is_empty() {
            format!("公开活动接口返回状态码 {status}")
        } else {
            format!("公开活动接口返回状态码 {status}：{}", text.trim())
        });
    }

    let payload = if text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
    };

    let success = payload
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    if !success {
        return Err(payload
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("公开活动接口返回失败")
            .to_string());
    }

    let data = payload.get("data").cloned().unwrap_or(payload);
    serde_json::from_value::<PublicActivityFeed>(data)
        .map_err(|error| format!("解析公开活动失败：{error}"))
}

fn select_dc_source_activity<'a>(
    active_statuses: &'a [PublicActivityItem],
    target_dc_source: &str,
) -> Option<&'a PublicActivityItem> {
    let normalized_target = target_dc_source.trim();
    if normalized_target.is_empty() {
        return None;
    }

    active_statuses
        .iter()
        .find(|item| item_dc_source(item) == Some(normalized_target))
}

fn map_activity_to_presence(item: &PublicActivityItem) -> DiscordPresencePayload {
    let details_source = item
        .status_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback_details(item));
    let details = normalize_presence_line(&details_source);

    let state = item
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_presence_line);
    let started_at_millis = item.started_at.as_deref().and_then(parse_started_at_millis);
    let summary = match state.as_deref() {
        Some(device) => format!("{details} · {device}"),
        None => details.clone(),
    };

    DiscordPresencePayload {
        details,
        state,
        started_at_millis,
        summary,
    }
}

fn fallback_details(item: &PublicActivityItem) -> String {
    let process_name = item
        .process_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let process_title = item
        .process_title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (process_title, process_name) {
        (Some(title), Some(name)) => format!("{title} | {name}"),
        (Some(title), None) => title.to_string(),
        (None, Some(name)) => name.to_string(),
        (None, None) => "Waken-Wa".to_string(),
    }
}

fn apply_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    payload: &DiscordPresencePayload,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, |client| {
        let mut activity_payload = activity::Activity::new().details(payload.details.clone());
        if let Some(state) = payload.state.as_deref() {
            activity_payload = activity_payload.state(state.to_string());
        }
        if let Some(started_at) = payload.started_at_millis {
            activity_payload =
                activity_payload.timestamps(activity::Timestamps::new().start(started_at));
        }
        client
            .set_activity(activity_payload)
            .map_err(|error| format!("更新 Discord 状态失败：{error}"))
    })
}

fn clear_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, |client| {
        client
            .clear_activity()
            .map_err(|error| format!("清空 Discord 状态失败：{error}"))
    })
}

fn with_discord_client<F>(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    mut action: F,
) -> Result<(), String>
where
    F: FnMut(&mut DiscordIpcClient) -> Result<(), String>,
{
    for _ in 0..2 {
        if client_slot.is_none() {
            let mut client = DiscordIpcClient::new(application_id);
            client
                .connect()
                .map_err(|error| format!("连接 Discord IPC 失败：{error}"))?;
            *client_slot = Some(client);
        }

        let Some(client) = client_slot.as_mut() else {
            continue;
        };

        match action(client) {
            Ok(()) => return Ok(()),
            Err(_) => {
                *client_slot = None;
            }
        }
    }

    Err("Discord IPC 当前不可用，请确认桌面端 Discord 已运行。".into())
}

fn validate_discord_presence_config(config: &ClientConfig) -> Result<(), String> {
    if config.base_url.trim().is_empty() {
        return Err("缺少站点地址，无法启动 Discord 状态同步。".into());
    }
    if config.discord_application_id.trim().is_empty() {
        return Err("缺少 Discord Application ID，无法启动 Discord 状态同步。".into());
    }
    if config.discord_source_id.trim().is_empty() {
        return Err("缺少 Discord 来源标识，无法筛选当前客户端的 Discord 状态来源。".into());
    }
    Ok(())
}

pub fn config_is_ready(config: &ClientConfig) -> bool {
    validate_discord_presence_config(config).is_ok()
}

fn normalize_presence_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn item_dc_source(item: &PublicActivityItem) -> Option<&str> {
    item.metadata
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("dc_source"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn parse_started_at_millis(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp_millis())
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

fn build_http_client(use_system_proxy: bool) -> Result<Client, String> {
    build_blocking_client(
        "waken-wa-tauri-discord-presence/0.1.0",
        Some(Duration::from_secs(15)),
        use_system_proxy,
    )
}

fn sleep_with_stop(duration: Duration, stop_flag: &Arc<AtomicBool>) {
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

fn error_backoff(consecutive_errors: u32) -> Duration {
    let multiplier = 2u64.saturating_pow(consecutive_errors.saturating_sub(1));
    Duration::from_millis(
        (SYNC_INTERVAL.as_millis() as u64)
            .saturating_mul(multiplier.max(1))
            .min(MAX_ERROR_BACKOFF_MS),
    )
}

fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
    let deadline = Instant::now() + WORKER_JOIN_WAIT_TIMEOUT;
    let handle = handle;

    while Instant::now() < deadline {
        if handle.is_finished() {
            let _ = handle.join();
            return Ok(());
        }
        thread::sleep(WORKER_JOIN_POLL_STEP);
    }

    Err(handle)
}

fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::{
        item_dc_source, map_activity_to_presence, select_dc_source_activity, PublicActivityItem,
    };
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
