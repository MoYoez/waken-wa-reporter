#[path = "realtime_reporter/logging.rs"]
mod logging;
#[path = "realtime_reporter/messages.rs"]
mod messages;
#[path = "realtime_reporter/payload.rs"]
mod payload;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use serde_json::{json, Value};

use crate::{
    backend_locale::BackendLocale,
    models::{
        ApiResult, ClientConfig, RealtimeReporterSnapshot, ReporterActivity, ReporterLogEntry,
    },
    platform::{
        get_foreground_snapshot_for_reporting, get_now_playing_artwork_for_reporting,
        get_now_playing_for_reporting, ForegroundSnapshot, MediaInfo,
    },
    runtime_utils::{now_iso_string, now_unix_millis, sleep_with_stop, wait_for_worker_exit},
};

use logging::{build_log_detail, fallback_text, localized_text, LogTextSpec};
use messages::reporter_worker_stopping;
use payload::{
    build_http_client, build_payload, parse_reporter_metadata, post_activity_blocking,
    validate_reporter_config, PostActivityResult,
};

const MAX_LOGS: usize = 120;
const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);
const MIN_MEDIA_POLL_INTERVAL: Duration = Duration::from_secs(5);

struct ReporterInner {
    running: bool,
    active_run_id: Option<u64>,
    logs: Vec<ReporterLogEntry>,
    current_activity: Option<ReporterActivity>,
    last_heartbeat_at: Option<String>,
    last_error: Option<String>,
    last_pending_approval_message: Option<String>,
    last_pending_approval_url: Option<String>,
    stop_flag: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

impl Default for ReporterInner {
    fn default() -> Self {
        Self {
            running: false,
            active_run_id: None,
            logs: Vec::new(),
            current_activity: None,
            last_heartbeat_at: None,
            last_error: None,
            last_pending_approval_message: None,
            last_pending_approval_url: None,
            stop_flag: None,
            worker: None,
        }
    }
}

pub struct ReporterRuntime {
    inner: Arc<Mutex<ReporterInner>>,
    sequence: AtomicU64,
}

impl ReporterRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReporterInner::default())),
            sequence: AtomicU64::new(1),
        }
    }

    pub fn snapshot(&self) -> RealtimeReporterSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        snapshot_from_inner(&inner)
    }

    pub fn start(
        &self,
        config: ClientConfig,
        locale: BackendLocale,
    ) -> Result<RealtimeReporterSnapshot, String> {
        validate_reporter_config(&config, locale)?;

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
                return Err(reporter_worker_stopping(locale));
            }
        }

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.stop_flag = Some(stop_flag.clone());
            inner.last_error = None;
            inner.last_pending_approval_message = None;
            inner.last_pending_approval_url = None;
        }

        self.push_log(
            "info",
            localized_text("reporterLogs.started.title", None, "实时上报已启动"),
            localized_text(
                "reporterLogs.started.detail",
                None,
                "后台轮询任务已经开始。",
            ),
            None,
        );

        let state = self.inner.clone();
        let sequence_seed = self.sequence.fetch_add(1, Ordering::SeqCst);
        let worker = thread::spawn(move || {
            run_reporter_loop(state, config, stop_flag, sequence_seed, run_id, locale);
        });

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = Some(worker);
        }

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> RealtimeReporterSnapshot {
        let worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
            inner.worker.take()
        };

        let (detail, stopped_cleanly) = if let Some(handle) = worker {
            if let Err(handle) =
                wait_for_worker_exit(handle, WORKER_JOIN_WAIT_TIMEOUT, WORKER_JOIN_POLL_STEP)
            {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
                (
                    localized_text(
                        "reporterLogs.stopped.pendingDetail",
                        None,
                        "停止信号已发送，后台线程会在当前操作完成后退出。",
                    ),
                    false,
                )
            } else {
                (
                    localized_text("reporterLogs.stopped.detail", None, "后台轮询任务已停止。"),
                    true,
                )
            }
        } else {
            (
                localized_text("reporterLogs.stopped.detail", None, "后台轮询任务已停止。"),
                true,
            )
        };

        if stopped_cleanly {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = None;
        }

        self.push_log(
            "warn",
            localized_text("reporterLogs.stopped.title", None, "实时上报已停止"),
            detail,
            None,
        );
        self.snapshot()
    }

    fn push_log(
        &self,
        level: &str,
        title: LogTextSpec,
        detail: LogTextSpec,
        payload: Option<Value>,
    ) {
        let id = format!(
            "{}-{}",
            now_unix_millis(),
            self.sequence.fetch_add(1, Ordering::SeqCst)
        );
        let entry = ReporterLogEntry {
            id,
            timestamp: now_iso_string(),
            level: level.to_string(),
            title: title.fallback,
            detail: detail.fallback,
            title_key: title.key.map(str::to_string),
            title_params: title.params,
            detail_key: detail.key.map(str::to_string),
            detail_params: detail.params,
            payload,
        };

        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.logs.insert(0, entry);
        if inner.logs.len() > MAX_LOGS {
            inner.logs.truncate(MAX_LOGS);
        }
    }
}

impl Drop for ReporterRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn snapshot_from_inner(inner: &ReporterInner) -> RealtimeReporterSnapshot {
    RealtimeReporterSnapshot {
        running: inner.running,
        logs: inner.logs.clone(),
        current_activity: inner.current_activity.clone(),
        last_heartbeat_at: inner.last_heartbeat_at.clone(),
        last_error: inner.last_error.clone(),
        last_pending_approval_message: inner.last_pending_approval_message.clone(),
        last_pending_approval_url: inner.last_pending_approval_url.clone(),
    }
}

fn run_reporter_loop(
    state: Arc<Mutex<ReporterInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    mut sequence_seed: u64,
    run_id: u64,
    locale: BackendLocale,
) {
    let client = match build_http_client(config.use_system_proxy, locale) {
        Ok(client) => client,
        Err(error) => {
            push_background_log(
                &state,
                &mut sequence_seed,
                "error",
                localized_text(
                    "reporterLogs.clientCreateFailed.title",
                    None,
                    "创建实时上报客户端失败",
                ),
                fallback_text(error.clone()),
                None,
            );
            mark_stopped(&state, Some(error), run_id);
            return;
        }
    };

    let metadata = match parse_reporter_metadata(&config.reporter_metadata_json, locale) {
        Ok(value) => value,
        Err(error) => {
            push_background_log(
                &state,
                &mut sequence_seed,
                "error",
                localized_text(
                    "reporterLogs.metadataInvalid.title",
                    None,
                    "实时上报元数据无效",
                ),
                fallback_text(error.clone()),
                None,
            );
            mark_stopped(&state, Some(error), run_id);
            return;
        }
    };

    let poll_interval = Duration::from_millis(config.poll_interval_ms.max(1_000));
    let heartbeat_interval = if config.heartbeat_interval_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(config.heartbeat_interval_ms))
    };

    let mut last_signature: Option<String> = None;
    let mut last_report_at: Option<SystemTime> = None;
    let mut consecutive_errors: u32 = 0;
    let mut last_media_error: Option<String> = None;
    let mut cached_media = MediaInfo::default();
    let mut last_media_poll_at: Option<SystemTime> = None;

    while !stop_flag.load(Ordering::SeqCst) {
        let mut iteration_had_error = false;

        let collect_foreground = config.report_foreground_app || config.report_window_title;
        let collect_media = config.report_media || config.report_play_source;

        let foreground = if collect_foreground {
            get_foreground_snapshot_for_reporting(
                config.report_foreground_app,
                config.report_window_title,
            )
        } else {
            Ok(ForegroundSnapshot::default())
        };

        match foreground {
            Ok(snapshot) => {
                let media = if collect_media {
                    let media_poll_interval = poll_interval.max(MIN_MEDIA_POLL_INTERVAL);
                    let should_poll_media = last_media_poll_at
                        .and_then(|last| SystemTime::now().duration_since(last).ok())
                        .map(|elapsed| elapsed >= media_poll_interval)
                        .unwrap_or(true);

                    if should_poll_media {
                        last_media_poll_at = Some(SystemTime::now());
                        let previous_media_signature = cached_media.signature_for_reporting(
                            config.report_media,
                            config.report_play_source,
                        );
                        cached_media = match get_now_playing_for_reporting(
                            config.report_media,
                            config.report_play_source,
                        ) {
                            Ok(mut media) => {
                                last_media_error = None;
                                if !previous_media_signature.is_empty()
                                    && previous_media_signature
                                        == media.signature_for_reporting(
                                            config.report_media,
                                            config.report_play_source,
                                        )
                                {
                                    media.cover_url = cached_media.cover_url.clone();
                                }
                                media
                            }
                            Err(error) => {
                                let should_log = last_media_error
                                    .as_ref()
                                    .map(|last| last != &error)
                                    .unwrap_or(true);
                                last_media_error = Some(error.clone());
                                if should_log {
                                    push_background_log(
                                        &state,
                                        &mut sequence_seed,
                                        "warn",
                                        localized_text(
                                            "reporterLogs.mediaReadFailed.title",
                                            None,
                                            "媒体信息读取失败",
                                        ),
                                        fallback_text(error.clone()),
                                        None,
                                    );
                                }
                                MediaInfo::default()
                            }
                        };
                    }

                    cached_media.clone()
                } else {
                    last_media_error = None;
                    cached_media = MediaInfo::default();
                    last_media_poll_at = None;
                    MediaInfo::default()
                };

                let signature = format!(
                    "{}\u{1e}{}\u{1e}{}",
                    snapshot.process_name,
                    snapshot.process_title,
                    media.signature_for_reporting(config.report_media, config.report_play_source)
                );
                let same_as_last = last_signature
                    .as_ref()
                    .map(|last| last == &signature)
                    .unwrap_or(false);

                let should_send = if same_as_last {
                    heartbeat_interval
                        .map(|interval| {
                            last_report_at
                                .and_then(|last| SystemTime::now().duration_since(last).ok())
                                .map(|elapsed| elapsed >= interval)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                } else {
                    true
                };

                if should_send {
                    let is_heartbeat = same_as_last;
                    let mut payload_media = media_with_artwork_for_payload(&media, &config);
                    if cached_media
                        .signature_for_reporting(config.report_media, config.report_play_source)
                        == payload_media
                            .signature_for_reporting(config.report_media, config.report_play_source)
                    {
                        cached_media.cover_url = payload_media.cover_url.clone();
                    }
                    let payload =
                        build_payload(&config, &snapshot, &payload_media, metadata.clone());
                    match post_activity_blocking(&client, &config, &payload, locale) {
                        Ok(PostActivityResult::Success {
                            media_cover_url,
                            response_text,
                        }) => {
                            // Update cached cover URL with the server-resolved URL for display
                            if let Some(ref server_url) = media_cover_url {
                                if !server_url.trim().is_empty() {
                                    payload_media.cover_url = server_url.clone();
                                    cached_media.cover_url = server_url.clone();
                                }
                            }
                            let detail =
                                build_log_detail(&snapshot, &payload_media, is_heartbeat, locale);
                            let response_payload =
                                serde_json::from_str::<Value>(&response_text).ok();
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "success",
                                if is_heartbeat {
                                    localized_text(
                                        "reporterLogs.activityHeartbeat.title",
                                        None,
                                        "活动心跳",
                                    )
                                } else {
                                    localized_text(
                                        "reporterLogs.activityReported.title",
                                        None,
                                        "活动已上报",
                                    )
                                },
                                detail,
                                response_payload,
                            );
                            update_snapshot(
                                &state,
                                &snapshot,
                                None,
                                Some(now_iso_string()),
                                media_cover_url,
                            );
                            last_signature = Some(signature);
                            last_report_at = Some(SystemTime::now());
                        }
                        Ok(PostActivityResult::PendingApproval {
                            message,
                            approval_url,
                            response_text,
                        }) => {
                            let detail = match approval_url {
                                Some(ref url) if !url.trim().is_empty() => {
                                    format!("{message}。请前往设备管理完成审核：{url}")
                                }
                                _ => message.clone(),
                            };
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "warn",
                                localized_text(
                                    "reporterLogs.pendingApproval.title",
                                    None,
                                    "设备待审核",
                                ),
                                match approval_url {
                                    Some(ref url) if !url.trim().is_empty() => localized_text(
                                        "reporterLogs.pendingApproval.withUrl",
                                        Some(json!({
                                            "message": message,
                                            "approvalUrl": url,
                                        })),
                                        detail,
                                    ),
                                    _ => localized_text(
                                        "reporterLogs.pendingApproval.withoutUrl",
                                        Some(json!({ "message": message })),
                                        detail,
                                    ),
                                },
                                serde_json::from_str::<Value>(&response_text).ok(),
                            );
                            set_pending_approval(
                                &state,
                                Some(message.clone()),
                                approval_url.clone(),
                            );
                            update_snapshot(&state, &snapshot, Some(message), None, None);
                            iteration_had_error = true;
                        }
                        Err(error) => {
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "error",
                                localized_text(
                                    "reporterLogs.reportFailed.title",
                                    None,
                                    "实时上报失败",
                                ),
                                fallback_text(error.clone()),
                                None,
                            );
                            update_snapshot(&state, &snapshot, Some(error), None, None);
                            iteration_had_error = true;
                        }
                    }
                }
            }
            Err(error) => {
                push_background_log(
                    &state,
                    &mut sequence_seed,
                    "error",
                    localized_text(
                        "reporterLogs.foregroundCaptureFailed.title",
                        None,
                        "采集前台窗口失败",
                    ),
                    fallback_text(error.clone()),
                    None,
                );
                set_last_error(&state, Some(error));
                iteration_had_error = true;
            }
        }

        if iteration_had_error {
            consecutive_errors = consecutive_errors.saturating_add(1);
        } else {
            consecutive_errors = 0;
        }

        let effective_sleep = if consecutive_errors > 1 {
            let backoff_ms =
                (poll_interval.as_millis() as u64).saturating_mul(1 << consecutive_errors.min(4));
            Duration::from_millis(backoff_ms.min(MAX_ERROR_BACKOFF_MS))
        } else {
            poll_interval
        };

        sleep_with_stop(effective_sleep, &stop_flag);
    }

    mark_stopped(&state, None, run_id);
}

fn media_with_artwork_for_payload(media: &MediaInfo, config: &ClientConfig) -> MediaInfo {
    let mut payload_media = media.clone();
    if !config.report_media
        || !config.report_media_artwork
        || payload_media.is_empty()
        || !payload_media.cover_url.trim().is_empty()
    {
        return payload_media;
    }

    let expected_signature =
        payload_media.signature_for_reporting(config.report_media, config.report_play_source);
    if expected_signature.is_empty() {
        return payload_media;
    }

    let Ok(artwork_media) = get_now_playing_artwork_for_reporting(config.report_play_source) else {
        return payload_media;
    };
    if artwork_media.signature_for_reporting(config.report_media, config.report_play_source)
        == expected_signature
    {
        payload_media.cover_url = artwork_media.cover_url;
    }

    payload_media
}

fn payload_for_log(payload: &crate::models::ActivityPayload) -> Option<Value> {
    let mut value = serde_json::to_value(payload).ok()?;
    omit_artwork_from_log_payload(&mut value);
    Some(value)
}

fn omit_artwork_from_log_payload(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("coverDataUrl");
            map.remove("cover_url");
            for child in map.values_mut() {
                omit_artwork_from_log_payload(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                omit_artwork_from_log_payload(item);
            }
        }
        _ => {}
    }
}

pub fn config_is_ready(config: &ClientConfig) -> bool {
    payload::config_is_ready(config)
}

fn update_snapshot(
    state: &Arc<Mutex<ReporterInner>>,
    snapshot: &ForegroundSnapshot,
    last_error: Option<String>,
    last_heartbeat_at: Option<String>,
    media_cover_url: Option<String>,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.current_activity = Some(ReporterActivity {
        process_name: snapshot.process_name.clone(),
        process_title: Some(snapshot.process_title.clone()),
        updated_at: Some(now_iso_string()),
        media_cover_url,
    });
    if let Some(error) = last_error {
        inner.last_error = Some(error);
    } else {
        inner.last_error = None;
        inner.last_pending_approval_message = None;
        inner.last_pending_approval_url = None;
    }
    if let Some(heartbeat_at) = last_heartbeat_at {
        inner.last_heartbeat_at = Some(heartbeat_at);
    }
}

fn set_last_error(state: &Arc<Mutex<ReporterInner>>, error: Option<String>) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.last_error = error;
}

fn set_pending_approval(
    state: &Arc<Mutex<ReporterInner>>,
    message: Option<String>,
    approval_url: Option<String>,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.last_pending_approval_message = message;
    inner.last_pending_approval_url = approval_url;
}

fn mark_stopped(state: &Arc<Mutex<ReporterInner>>, error: Option<String>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.running = false;
    inner.active_run_id = None;
    inner.stop_flag = None;
    if error.is_some() {
        inner.last_error = error;
    }
}

fn push_background_log(
    state: &Arc<Mutex<ReporterInner>>,
    sequence: &mut u64,
    level: &str,
    title: LogTextSpec,
    detail: LogTextSpec,
    payload: Option<Value>,
) {
    let entry = ReporterLogEntry {
        id: format!("{}-{}", now_unix_millis(), *sequence),
        timestamp: now_iso_string(),
        level: level.to_string(),
        title: title.fallback,
        detail: detail.fallback,
        title_key: title.key.map(str::to_string),
        title_params: title.params,
        detail_key: detail.key.map(str::to_string),
        detail_params: detail.params,
        payload,
    };
    *sequence += 1;

    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.logs.insert(0, entry);
    if inner.logs.len() > MAX_LOGS {
        inner.logs.truncate(MAX_LOGS);
    }
}

pub fn snapshot_result(runtime: &ReporterRuntime) -> ApiResult<RealtimeReporterSnapshot> {
    ApiResult::success(200, runtime.snapshot())
}
