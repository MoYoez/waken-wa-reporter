use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub fn default_device_name() -> String {
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Waken-Wa Client".into())
}

pub fn effective_device_name(value: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        default_device_name()
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
    "{\n  \"source\": \"waken-wa-client\"\n}".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub realtime_reporter: bool,
    pub tray: bool,
    pub platform_self_test: bool,
}

#[cfg(desktop)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: true,
        tray: true,
        platform_self_test: true,
    }
}

#[cfg(mobile)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
    }
}

#[cfg(not(any(desktop, mobile)))]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
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

    pub fn failure(status: u16, message: impl Into<String>, details: Option<Value>) -> Self {
        Self {
            success: false,
            status,
            data: None,
            error: Some(ApiError {
                status,
                message: message.into(),
                details,
            }),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformProbeResult {
    pub success: bool,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub guidance: Vec<String>,
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
