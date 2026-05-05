mod app_state;
mod helpers;
mod http;
mod platform;
#[cfg(desktop)]
mod runtime;

use serde_json::Value;
use tauri::AppHandle;
#[cfg(desktop)]
use tauri::State;

use crate::models::{
    ActivityPayload, ApiResult, AppStatePayload, ClientCapabilities, ClientConfig,
    DiscordPresenceSnapshot, ExistingReporterConfig, ImportedIntegrationConfig,
    InspirationEntryCreateInput, PlatformSelfTestResult, RealtimeReporterSnapshot,
};
#[cfg(desktop)]
use crate::{discord_presence::DiscordPresenceRuntime, realtime_reporter::ReporterRuntime};

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    app_state::load_app_state(app)
}

#[tauri::command]
pub fn save_app_state(
    app: AppHandle,
    payload: Value,
) -> Result<ApiResult<AppStatePayload>, String> {
    app_state::save_app_state(app, payload)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    app_state::restart_app(app)
}

#[tauri::command]
pub fn parse_imported_integration_config(
    input: String,
) -> Result<ApiResult<ImportedIntegrationConfig>, String> {
    app_state::parse_imported_integration_config(input)
}

#[tauri::command]
pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    app_state::get_client_capabilities()
}

#[cfg(desktop)]
#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    runtime::hide_to_tray(app)
}

#[tauri::command]
pub async fn submit_activity_report(
    config: ClientConfig,
    payload: ActivityPayload,
) -> Result<ApiResult<Value>, String> {
    http::submit_activity_report(config, payload).await
}

#[tauri::command]
pub async fn get_public_activity_feed(config: ClientConfig) -> Result<ApiResult<Value>, String> {
    http::get_public_activity_feed(config).await
}

#[tauri::command]
pub async fn list_inspiration_entries(
    config: ClientConfig,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<ApiResult<Value>, String> {
    http::list_inspiration_entries(config, limit, offset).await
}

#[tauri::command]
pub async fn probe_connectivity(config: ClientConfig) -> Result<ApiResult<Value>, String> {
    http::probe_connectivity(config).await
}

#[tauri::command]
pub async fn create_inspiration_entry(
    config: ClientConfig,
    input: InspirationEntryCreateInput,
) -> Result<ApiResult<Value>, String> {
    http::create_inspiration_entry(config, input).await
}

#[tauri::command]
pub async fn upload_inspiration_asset(
    config: ClientConfig,
    image_data_url: String,
) -> Result<ApiResult<Value>, String> {
    http::upload_inspiration_asset(config, image_data_url).await
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::start_realtime_reporter(app, reporter, config)
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_realtime_reporter(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::stop_realtime_reporter(reporter)
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::get_realtime_reporter_snapshot(reporter)
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::start_discord_presence_sync(app, discord_presence_runtime, config)
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_discord_presence_sync(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::stop_discord_presence_sync(discord_presence_runtime)
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::get_discord_presence_snapshot(discord_presence_runtime)
}

#[tauri::command]
pub async fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    platform::run_platform_self_test().await
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<ApiResult<bool>, String> {
    platform::request_accessibility_permission()
}

#[tauri::command]
pub fn discover_existing_reporter_config(
    app: AppHandle,
) -> Result<ApiResult<ExistingReporterConfig>, String> {
    app_state::discover_existing_reporter_config(app)
}
