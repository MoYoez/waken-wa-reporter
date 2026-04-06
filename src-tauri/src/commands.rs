use serde_json::json;
use tauri::{AppHandle, State};

use crate::{
    http_client::request_json,
    import_config::parse_import_payload,
    models::{
        default_client_capabilities, ActivityPayload, ApiResult, AppStatePayload, ClientCapabilities,
        ClientConfig, ExistingReporterConfig, ImportedIntegrationConfig, InspirationEntryCreateInput,
        PlatformSelfTestResult,
    },
    platform,
    reporter_config, state_store,
};

#[cfg(desktop)]
use crate::realtime_reporter::{snapshot_result, ReporterRuntime};
#[cfg(desktop)]
use crate::tray;

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

#[tauri::command]
pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state_store::save_app_state(&app, &payload)
}

#[tauri::command]
pub fn parse_imported_integration_config(
    input: String,
) -> Result<ImportedIntegrationConfig, String> {
    parse_import_payload(&input)
}

#[tauri::command]
pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    Ok(ApiResult::success(200, default_client_capabilities()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    tray::hide_main_window(&app)
}

#[tauri::command]
pub async fn submit_activity_report(
    config: ClientConfig,
    payload: ActivityPayload,
) -> Result<ApiResult<serde_json::Value>, String> {
    let generated_hash_key = if payload.generated_hash_key.trim().is_empty() {
        config.generated_hash_key.trim().to_string()
    } else {
        payload.generated_hash_key.trim().to_string()
    };

    let process_name = payload.process_name.trim().to_string();

    let body = json!({
        "generatedHashKey": generated_hash_key,
        "process_name": process_name,
        "device": payload.device.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
        "process_title": payload.process_title.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
        "persist_minutes": payload.persist_minutes,
        "battery_level": payload.battery_level,
        "is_charging": payload.is_charging,
        "device_type": payload.device_type.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
        "push_mode": payload.push_mode.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
        "metadata": payload.metadata,
    });

    Ok(request_json(
        &config.base_url,
        "/api/activity",
        Some(&config.api_token),
        reqwest::Method::POST,
        Some(body),
    )
    .await)
}

#[tauri::command]
pub async fn get_public_activity_feed(
    config: ClientConfig,
) -> Result<ApiResult<serde_json::Value>, String> {
    Ok(request_json(
        &config.base_url,
        "/api/activity?public=1",
        None,
        reqwest::Method::GET,
        None,
    )
    .await)
}

#[tauri::command]
pub async fn list_inspiration_entries(
    config: ClientConfig,
) -> Result<ApiResult<serde_json::Value>, String> {
    Ok(request_json(
        &config.base_url,
        "/api/inspiration/entries",
        None,
        reqwest::Method::GET,
        None,
    )
    .await)
}

#[tauri::command]
pub async fn create_inspiration_entry(
    config: ClientConfig,
    input: InspirationEntryCreateInput,
) -> Result<ApiResult<serde_json::Value>, String> {
    let body = json!({
        "title": input.title.trim(),
        "content": input.content.trim(),
        "contentLexical": input.content_lexical,
        "imageDataUrl": input.image_data_url,
        "generatedHashKey": input.generated_hash_key,
        "attachCurrentStatus": input.attach_current_status,
        "preComputedStatusSnapshot": input.pre_computed_status_snapshot,
        "attachStatusDeviceHash": input.attach_status_device_hash,
        "attachStatusActivityKey": input.attach_status_activity_key,
        "attachStatusIncludeDeviceInfo": input.attach_status_include_device_info,
    });

    Ok(request_json(
        &config.base_url,
        "/api/inspiration/entries",
        Some(&config.api_token),
        reqwest::Method::POST,
        Some(body),
    )
    .await)
}

#[tauri::command]
pub async fn upload_inspiration_asset(
    config: ClientConfig,
    image_data_url: String,
) -> Result<ApiResult<serde_json::Value>, String> {
    let body = json!({
        "imageDataUrl": image_data_url,
        "generatedHashKey": config.generated_hash_key.trim(),
    });

    Ok(request_json(
        &config.base_url,
        "/api/inspiration/assets",
        Some(&config.api_token),
        reqwest::Method::POST,
        Some(body),
    )
    .await)
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_realtime_reporter(
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    let snapshot = reporter.start(config)?;
    Ok(ApiResult::success(200, snapshot))
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_realtime_reporter(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    Ok(ApiResult::success(200, reporter.stop()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    Ok(snapshot_result(&reporter))
}

#[tauri::command]
pub fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    Ok(ApiResult::success(200, platform::run_self_test()))
}

#[tauri::command]
pub fn discover_existing_reporter_config(
    app: AppHandle,
) -> Result<ApiResult<ExistingReporterConfig>, String> {
    let config = reporter_config::discover_existing_reporter_config(&app)?;
    Ok(ApiResult::success(200, config))
}
