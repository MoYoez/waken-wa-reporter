use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub fn system_device_name() -> String {
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Waken-Wa Client".into())
}

pub fn default_device_name() -> String {
    String::new()
}

pub fn effective_device_name(value: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        system_device_name()
    } else {
        normalized.to_string()
    }
}

pub fn default_device_type() -> String {
    "desktop".into()
}

pub fn default_push_mode() -> String {
    "realtime".into()
}

pub fn default_use_system_proxy() -> bool {
    true
}

pub fn default_poll_interval_ms() -> u64 {
    5_000
}

pub fn default_heartbeat_interval_ms() -> u64 {
    60_000
}

pub fn default_reporter_metadata_json() -> String {
    String::new()
}

pub fn default_report_foreground_app() -> bool {
    true
}

pub fn default_report_window_title() -> bool {
    true
}

pub fn default_report_media() -> bool {
    true
}

pub fn default_report_play_source() -> bool {
    true
}

pub fn default_discord_application_id() -> String {
    String::new()
}

pub fn default_discord_source_id() -> String {
    String::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub realtime_reporter: bool,
    pub tray: bool,
    pub platform_self_test: bool,
    pub discord_presence: bool,
    pub autostart: bool,
}

#[cfg(desktop)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: true,
        tray: true,
        platform_self_test: true,
        discord_presence: true,
        autostart: true,
    }
}

#[cfg(mobile)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}

#[cfg(not(any(desktop, mobile)))]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_token: String,
    #[serde(default)]
    pub generated_hash_key: String,
    #[serde(default = "default_use_system_proxy")]
    pub use_system_proxy: bool,
    #[serde(default = "default_device_name")]
    pub device: String,
    #[serde(default = "default_device_type")]
    pub device_type: String,
    #[serde(default = "default_push_mode")]
    pub push_mode: String,
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_heartbeat_interval_ms")]
    pub heartbeat_interval_ms: u64,
    #[serde(default = "default_reporter_metadata_json")]
    pub reporter_metadata_json: String,
    #[serde(default)]
    pub reporter_enabled: bool,
    #[serde(default = "default_report_foreground_app")]
    pub report_foreground_app: bool,
    #[serde(default = "default_report_window_title")]
    pub report_window_title: bool,
    #[serde(default = "default_report_media")]
    pub report_media: bool,
    #[serde(default = "default_report_play_source")]
    pub report_play_source: bool,
    #[serde(default)]
    pub discord_enabled: bool,
    #[serde(default = "default_discord_application_id")]
    pub discord_application_id: String,
    #[serde(default = "default_discord_source_id")]
    pub discord_source_id: String,
    #[serde(default)]
    pub launch_on_startup: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_token: String::new(),
            generated_hash_key: String::new(),
            use_system_proxy: default_use_system_proxy(),
            device: default_device_name(),
            device_type: default_device_type(),
            push_mode: default_push_mode(),
            poll_interval_ms: default_poll_interval_ms(),
            heartbeat_interval_ms: default_heartbeat_interval_ms(),
            reporter_metadata_json: default_reporter_metadata_json(),
            reporter_enabled: false,
            report_foreground_app: default_report_foreground_app(),
            report_window_title: default_report_window_title(),
            report_media: default_report_media(),
            report_play_source: default_report_play_source(),
            discord_enabled: false,
            discord_application_id: default_discord_application_id(),
            discord_source_id: default_discord_source_id(),
            launch_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPreset {
    pub process_name: String,
    pub process_title: Option<String>,
    pub media_title: Option<String>,
    pub media_singer: Option<String>,
    pub last_used_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppStatePayload {
    #[serde(default)]
    pub config: ClientConfig,
    #[serde(default)]
    pub recent_presets: Vec<RecentPreset>,
    #[serde(default)]
    pub onboarding_dismissed: bool,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub reporter_config_prompt_handled: bool,
    #[serde(default)]
    pub verified_generated_hash_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPayload {
    #[serde(rename = "generatedHashKey", alias = "generated_hash_key")]
    pub generated_hash_key: String,
    #[serde(rename = "process_name", alias = "processName")]
    pub process_name: String,
    #[serde(rename = "device", default)]
    pub device: Option<String>,
    #[serde(rename = "process_title", alias = "processTitle", default)]
    pub process_title: Option<String>,
    #[serde(rename = "persist_minutes", alias = "persistMinutes", default)]
    pub persist_minutes: Option<i64>,
    #[serde(rename = "battery_level", alias = "batteryLevel", default)]
    pub battery_level: Option<i64>,
    #[serde(rename = "is_charging", alias = "isCharging", default)]
    pub is_charging: Option<bool>,
    #[serde(rename = "device_type", alias = "deviceType", default)]
    pub device_type: Option<String>,
    #[serde(rename = "push_mode", alias = "pushMode", default)]
    pub push_mode: Option<String>,
    #[serde(rename = "metadata", default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationEntryCreateInput {
    pub title: String,
    pub content: String,
    pub content_lexical: Option<String>,
    pub image_data_url: Option<String>,
    pub generated_hash_key: Option<String>,
    pub attach_current_status: Option<bool>,
    pub pre_computed_status_snapshot: Option<String>,
    pub attach_status_device_hash: Option<String>,
    pub attach_status_activity_key: Option<String>,
    pub attach_status_include_device_info: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedIntegrationConfig {
    pub report_endpoint: Option<String>,
    pub token: Option<String>,
    pub token_name: Option<String>,
    pub device_name: Option<String>,
    pub raw: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub status: u16,
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResult<T> {
    pub success: bool,
    pub status: u16,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResult<T> {
    pub fn success(status: u16, data: T) -> Self {
        Self {
            success: true,
            status,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure_localized<S>(
        status: u16,
        code: Option<S>,
        message: impl Into<String>,
        params: Option<Value>,
        details: Option<Value>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            success: false,
            status,
            data: None,
            error: Some(ApiError {
                status,
                message: message.into(),
                code: code.map(Into::into),
                params,
                details,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedTextEntry {
    pub text: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReporterActivity {
    #[serde(default)]
    pub process_name: String,
    #[serde(default)]
    pub process_title: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterLogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub title: String,
    pub detail: String,
    #[serde(default)]
    pub title_key: Option<String>,
    #[serde(default)]
    pub title_params: Option<Value>,
    #[serde(default)]
    pub detail_key: Option<String>,
    #[serde(default)]
    pub detail_params: Option<Value>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeReporterSnapshot {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub logs: Vec<ReporterLogEntry>,
    #[serde(default)]
    pub current_activity: Option<ReporterActivity>,
    #[serde(default)]
    pub last_heartbeat_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_pending_approval_message: Option<String>,
    #[serde(default)]
    pub last_pending_approval_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordPresenceSnapshot {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub current_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformProbeResult {
    pub success: bool,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub guidance: Vec<String>,
    #[serde(default)]
    pub summary_key: Option<String>,
    #[serde(default)]
    pub summary_params: Option<Value>,
    #[serde(default)]
    pub detail_key: Option<String>,
    #[serde(default)]
    pub detail_params: Option<Value>,
    #[serde(default)]
    pub guidance_entries: Vec<LocalizedTextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSelfTestResult {
    pub platform: String,
    pub foreground: PlatformProbeResult,
    pub window_title: PlatformProbeResult,
    pub media: PlatformProbeResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingReporterConfig {
    pub found: bool,
    pub path: Option<String>,
    pub config: Option<ClientConfig>,
}
