use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Map, Value};

use crate::{
    http_client::build_blocking_client,
    models::{
        effective_device_name, ActivityPayload, ApiResult, ClientConfig, RealtimeReporterSnapshot,
        ReporterActivity, ReporterLogEntry,
    },
    platform::{get_foreground_snapshot, get_now_playing, ForegroundSnapshot, MediaInfo},
};

const MAX_LOGS: usize = 120;
const MAX_ERROR_BACKOFF_MS: u64 = 30_000;

enum PostActivityResult {
    Success,
    PendingApproval {
        message: String,
        approval_url: Option<String>,
    },
}

#[derive(Default)]
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

    pub fn start(&self, config: ClientConfig) -> Result<RealtimeReporterSnapshot, String> {
        validate_reporter_config(&config)?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let run_id = self.sequence.fetch_add(1, Ordering::SeqCst);

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.running {
                return Ok(snapshot_from_inner(&inner));
            }

            if let Some(old_flag) = inner.stop_flag.take() {
                old_flag.store(true, Ordering::SeqCst);
            }

            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.stop_flag = Some(stop_flag.clone());
            inner.last_error = None;
            inner.last_pending_approval_message = None;
            inner.last_pending_approval_url = None;
        }

        self.push_log("info", "实时上报已启动", "后台轮询任务已经开始。", None);

        let state = self.inner.clone();
        let sequence_seed = self.sequence.fetch_add(1, Ordering::SeqCst);

        thread::spawn(move || {
            run_reporter_loop(state, config, stop_flag, sequence_seed, run_id);
        });

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> RealtimeReporterSnapshot {
        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
        }
        self.push_log("warn", "实时上报已停止", "后台轮询任务已停止。", None);
        self.snapshot()
    }

    fn push_log(&self, level: &str, title: &str, detail: &str, payload: Option<Value>) {
        let id = format!(
            "{}-{}",
            now_unix_millis(),
            self.sequence.fetch_add(1, Ordering::SeqCst)
        );
        let entry = ReporterLogEntry {
            id,
            timestamp: now_iso_string(),
            level: level.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
            payload,
        };
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.logs.insert(0, entry);
        if inner.logs.len() > MAX_LOGS {
            inner.logs.truncate(MAX_LOGS);
        }
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
) {
    let client = match build_http_client(config.use_system_proxy) {
        Ok(client) => client,
        Err(error) => {
            push_background_log(
                &state,
                &mut sequence_seed,
                "error",
                "创建实时上报客户端失败",
                &error,
                None,
            );
            mark_stopped(&state, Some(error), run_id);
            return;
        }
    };

    let metadata = match parse_reporter_metadata(&config.reporter_metadata_json) {
        Ok(value) => value,
        Err(error) => {
            push_background_log(
                &state,
                &mut sequence_seed,
                "error",
                "实时上报元数据无效",
                &error,
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

    while !stop_flag.load(Ordering::SeqCst) {
        let mut iteration_had_error = false;

        match get_foreground_snapshot() {
            Ok(snapshot) => {
                let media = match get_now_playing() {
                    Ok(media) => media,
                    Err(error) => {
                        push_background_log(
                            &state,
                            &mut sequence_seed,
                            "warn",
                            "媒体信息读取失败",
                            &error,
                            None,
                        );
                        MediaInfo::default()
                    }
                };

                let signature = format!(
                    "{}\u{1e}{}\u{1e}{}",
                    snapshot.process_name,
                    snapshot.process_title,
                    media.signature()
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
                    let payload = build_payload(&config, &snapshot, &media, metadata.clone());
                    match post_activity_blocking(&client, &config, &payload) {
                        Ok(PostActivityResult::Success) => {
                            let detail = build_log_detail(&snapshot, &media, is_heartbeat);
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "success",
                                if is_heartbeat {
                                    "活动心跳"
                                } else {
                                    "活动已上报"
                                },
                                &detail,
                                serde_json::to_value(&payload).ok(),
                            );
                            update_snapshot(&state, &snapshot, None, Some(now_iso_string()));
                            last_signature = Some(signature);
                            last_report_at = Some(SystemTime::now());
                        }
                        Ok(PostActivityResult::PendingApproval {
                            message,
                            approval_url,
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
                                "设备待审核",
                                &detail,
                                serde_json::to_value(&payload).ok(),
                            );
                            set_pending_approval(
                                &state,
                                Some(message.clone()),
                                approval_url.clone(),
                            );
                            update_snapshot(&state, &snapshot, Some(message), None);
                            iteration_had_error = true;
                        }
                        Err(error) => {
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "error",
                                "实时上报失败",
                                &error,
                                None,
                            );
                            update_snapshot(&state, &snapshot, Some(error), None);
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
                    "采集前台窗口失败",
                    &error,
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

fn validate_reporter_config(config: &ClientConfig) -> Result<(), String> {
    if config.base_url.trim().is_empty() {
        return Err("缺少站点地址，无法启动实时上报。".into());
    }
    if config.api_token.trim().is_empty() {
        return Err("缺少 API Token，无法启动实时上报。".into());
    }
    if config.generated_hash_key.trim().is_empty() {
        return Err("缺少 GeneratedHashKey，无法启动实时上报。".into());
    }
    Ok(())
}

pub fn config_is_ready(config: &ClientConfig) -> bool {
    validate_reporter_config(config).is_ok()
}

fn build_log_detail(
    snapshot: &ForegroundSnapshot,
    media: &MediaInfo,
    is_heartbeat: bool,
) -> String {
    let action = if is_heartbeat {
        "心跳上报"
    } else {
        "活动上报"
    };
    if media.is_empty() {
        format!(
            "{action}：{} / {}",
            snapshot.process_name, snapshot.process_title
        )
    } else {
        format!(
            "{action}：{} / {} | 媒体：{}",
            snapshot.process_name,
            snapshot.process_title,
            media.summary()
        )
    }
}

fn update_snapshot(
    state: &Arc<Mutex<ReporterInner>>,
    snapshot: &ForegroundSnapshot,
    last_error: Option<String>,
    last_heartbeat_at: Option<String>,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.current_activity = Some(ReporterActivity {
        process_name: snapshot.process_name.clone(),
        process_title: Some(snapshot.process_title.clone()),
        updated_at: Some(now_iso_string()),
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
    title: &str,
    detail: &str,
    payload: Option<Value>,
) {
    let entry = ReporterLogEntry {
        id: format!("{}-{}", now_unix_millis(), *sequence),
        timestamp: now_iso_string(),
        level: level.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        payload,
    };
    *sequence += 1;

    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.logs.insert(0, entry);
    if inner.logs.len() > MAX_LOGS {
        inner.logs.truncate(MAX_LOGS);
    }
}

fn build_http_client(use_system_proxy: bool) -> Result<Client, String> {
    build_blocking_client(
        "waken-wa-tauri-reporter/0.1.0",
        Some(Duration::from_secs(15)),
        use_system_proxy,
    )
}

fn parse_reporter_metadata(input: &str) -> Result<Map<String, Value>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Map::new());
    }

    match serde_json::from_str::<Value>(trimmed) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("实时上报元数据必须是 JSON 对象。".into()),
        Err(error) => Err(format!("解析实时上报元数据失败：{error}")),
    }
}

fn build_payload(
    config: &ClientConfig,
    snapshot: &ForegroundSnapshot,
    media: &MediaInfo,
    mut metadata: Map<String, Value>,
) -> ActivityPayload {
    if !metadata.contains_key("source") {
        metadata.insert("source".into(), Value::String("waken-wa-client".into()));
    }

    if let Some(media_map) = media.as_metadata_map() {
        metadata.insert("media".into(), Value::Object(media_map));
    }
    if !media.source_app_id.trim().is_empty() && !metadata.contains_key("play_source") {
        metadata.insert(
            "play_source".into(),
            Value::String(media.source_app_id.trim().to_string()),
        );
    }

    ActivityPayload {
        generated_hash_key: config.generated_hash_key.trim().to_string(),
        process_name: snapshot.process_name.clone(),
        device: Some(effective_device_name(&config.device)),
        process_title: Some(snapshot.process_title.clone()).filter(|value| !value.is_empty()),
        persist_minutes: None,
        battery_level: None,
        is_charging: None,
        device_type: Some(config.device_type.trim().to_string()).filter(|value| !value.is_empty()),
        push_mode: Some(config.push_mode.trim().to_string()).filter(|value| !value.is_empty()),
        metadata: Some(Value::Object(metadata)),
    }
}

fn post_activity_blocking(
    client: &Client,
    config: &ClientConfig,
    payload: &ActivityPayload,
) -> Result<PostActivityResult, String> {
    let body =
        serde_json::to_value(payload).map_err(|error| format!("序列化上报数据失败：{error}"))?;
    let url = format!("{}/api/activity", config.base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", config.api_token.trim()))
        .json(&body)
        .send()
        .map_err(|error| format!("请求失败：{error}"))?;

    let status = response.status().as_u16();
    let text = response.text().unwrap_or_default();
    if status >= 400 {
        return Err(if text.trim().is_empty() {
            format!("服务端返回状态码 {status}")
        } else {
            format!("服务端返回状态码 {status}：{}", text.trim())
        });
    }

    let parsed = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }));
    let success = parsed
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    if status == 202
        && parsed
            .get("pending")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        let message = parsed
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| parsed.get("message").and_then(Value::as_str))
            .unwrap_or("设备待后台审核后可用")
            .to_string();
        let approval_url = parsed
            .get("approvalUrl")
            .and_then(Value::as_str)
            .map(str::to_string);
        return Ok(PostActivityResult::PendingApproval {
            message,
            approval_url,
        });
    }

    if !success {
        return Err(parsed
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| parsed.get("message").and_then(Value::as_str))
            .unwrap_or("服务端返回失败")
            .to_string());
    }

    Ok(PostActivityResult::Success)
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

fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

pub fn snapshot_result(runtime: &ReporterRuntime) -> ApiResult<RealtimeReporterSnapshot> {
    ApiResult::success(200, runtime.snapshot())
}
