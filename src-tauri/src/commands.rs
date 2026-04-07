use serde_json::json;
use tauri::{AppHandle, State};

use crate::{
    http_client::{request_json, request_json_payload},
    import_config::parse_import_payload,
    models::{
        default_client_capabilities, effective_device_name, ActivityPayload, ApiResult,
        AppStatePayload, ClientCapabilities, ClientConfig, ExistingReporterConfig,
        ImportedIntegrationConfig, InspirationEntryCreateInput, PlatformSelfTestResult,
    },
    platform, reporter_config, state_store,
};

const MAX_IMAGE_DATA_URL_BYTES: usize = 7 * 1024 * 1024;

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

    let resolved_device = payload
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| effective_device_name(&config.device));

    let body = json!({
        "generatedHashKey": generated_hash_key,
        "process_name": process_name,
        "device": resolved_device,
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
        config.use_system_proxy,
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
        config.use_system_proxy,
        reqwest::Method::GET,
        None,
    )
    .await)
}

#[tauri::command]
pub async fn list_inspiration_entries(
    config: ClientConfig,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<ApiResult<serde_json::Value>, String> {
    let mut params = Vec::new();
    if let Some(limit) = limit {
        params.push(format!("limit={limit}"));
    }
    if let Some(offset) = offset {
        params.push(format!("offset={offset}"));
    }

    let path = if params.is_empty() {
        "/api/inspiration/entries".to_string()
    } else {
        format!("/api/inspiration/entries?{}", params.join("&"))
    };

    Ok(request_json_payload(
        &config.base_url,
        &path,
        None,
        config.use_system_proxy,
        reqwest::Method::GET,
        None,
    )
    .await)
}

#[tauri::command]
pub async fn probe_connectivity(
    config: ClientConfig,
) -> Result<ApiResult<serde_json::Value>, String> {
    let body = json!({
        "generatedHashKey": config.generated_hash_key.trim(),
        "device": effective_device_name(&config.device),
        "deviceType": config.device_type.trim(),
    });

    Ok(request_json(
        &config.base_url,
        "/api/activity/verify",
        Some(&config.api_token),
        config.use_system_proxy,
        reqwest::Method::POST,
        Some(body),
    )
    .await)
}

#[tauri::command]
pub async fn create_inspiration_entry(
    config: ClientConfig,
    input: InspirationEntryCreateInput,
) -> Result<ApiResult<serde_json::Value>, String> {
    validate_image_data_url(input.image_data_url.as_deref())?;

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
        config.use_system_proxy,
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
    validate_image_data_url(Some(&image_data_url))?;

    let body = json!({
        "imageDataUrl": image_data_url,
        "generatedHashKey": config.generated_hash_key.trim(),
    });

    Ok(request_json(
        &config.base_url,
        "/api/inspiration/assets",
        Some(&config.api_token),
        config.use_system_proxy,
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

fn validate_image_data_url(value: Option<&str>) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    if value.len() > MAX_IMAGE_DATA_URL_BYTES {
        return Err("图片数据过大，请选择更小的图片。".into());
    }

    if !value.starts_with("data:image/") {
        return Err("图片格式无效，请重新选择图片。".into());
    }

    Ok(())
}
