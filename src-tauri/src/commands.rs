use serde_json::{json, Map, Value};
use tauri::{AppHandle, State};

use crate::{
    backend_locale::load_locale,
    discord_presence::DiscordPresenceRuntime,
    http_client::{request_json, request_json_payload},
    import_config::parse_import_payload,
    models::{
        default_client_capabilities, effective_device_name, ActivityPayload, ApiResult,
        AppStatePayload, ClientCapabilities, ClientConfig, DiscordPresenceSnapshot,
        ExistingReporterConfig, ImportedIntegrationConfig, InspirationEntryCreateInput,
        PlatformSelfTestResult,
    },
    platform, reporter_config, state_store,
};

const MAX_IMAGE_DATA_URL_BYTES: usize = 7 * 1024 * 1024;
const MAX_IMPORT_INPUT_BYTES: usize = 512 * 1024;

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
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    app.request_restart();
    Ok(())
}

#[tauri::command]
pub fn parse_imported_integration_config(
    input: String,
) -> Result<ApiResult<ImportedIntegrationConfig>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.importConfigEmpty".to_string()),
            "请先粘贴 Base64 或 JSON 配置。",
            None,
            None,
        ));
    }

    if trimmed.len() > MAX_IMPORT_INPUT_BYTES {
        return Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.importConfigTooLarge".to_string()),
            format!(
                "输入内容过大（{} 字节），最大允许 {} 字节。",
                trimmed.len(),
                MAX_IMPORT_INPUT_BYTES
            ),
            Some(json!({
                "size": trimmed.len(),
                "maxSize": MAX_IMPORT_INPUT_BYTES,
            })),
            None,
        ));
    }

    match parse_import_payload(&input) {
        Ok(parsed) => Ok(ApiResult::success(200, parsed)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.importConfigInvalid".to_string()),
            error,
            None,
            None,
        )),
    }
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
        "metadata": with_dc_source_metadata(payload.metadata, &config.discord_source_id),
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
    if let Err(error) = validate_image_data_url(input.image_data_url.as_deref()) {
        let ValidationError {
            code,
            message,
            params,
        } = error;
        return Ok(ApiResult::failure_localized(
            400,
            Some(code.to_string()),
            message,
            params,
            None,
        ));
    }

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
    if let Err(error) = validate_image_data_url(Some(&image_data_url)) {
        let ValidationError {
            code,
            message,
            params,
        } = error;
        return Ok(ApiResult::failure_localized(
            400,
            Some(code.to_string()),
            message,
            params,
            None,
        ));
    }

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
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    match reporter.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            reporter_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
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

#[cfg(desktop)]
#[tauri::command]
pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    match discord_presence_runtime.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            discord_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_discord_presence_sync(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.stop()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.snapshot()))
}

#[tauri::command]
pub fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    Ok(ApiResult::success(200, platform::run_self_test()))
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<ApiResult<bool>, String> {
    match platform::request_accessibility_permission() {
        Ok(granted) => Ok(ApiResult::success(200, granted)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.accessibilityPermissionUnsupported".to_string()),
            error,
            None,
            None,
        )),
    }
}

#[tauri::command]
pub fn discover_existing_reporter_config(
    app: AppHandle,
) -> Result<ApiResult<ExistingReporterConfig>, String> {
    let config = reporter_config::discover_existing_reporter_config(&app)?;
    Ok(ApiResult::success(200, config))
}

struct ValidationError {
    code: &'static str,
    message: String,
    params: Option<Value>,
}

fn validate_image_data_url(value: Option<&str>) -> Result<(), ValidationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    if value.len() > MAX_IMAGE_DATA_URL_BYTES {
        return Err(ValidationError {
            code: "backendErrors.imageTooLarge",
            message: "图片数据过大，请选择更小的图片。".into(),
            params: Some(json!({ "maxBytes": MAX_IMAGE_DATA_URL_BYTES })),
        });
    }

    if !value.starts_with("data:image/") {
        return Err(ValidationError {
            code: "backendErrors.imageInvalidType",
            message: "图片格式无效，请重新选择图片。".into(),
            params: None,
        });
    }

    Ok(())
}

fn reporter_start_error_code(error: &str) -> Option<&'static str> {
    if error.contains("缺少站点地址") || error.contains("Site URL is required") {
        Some("backendErrors.reporterConfigBaseUrlMissing")
    } else if error.contains("缺少 API Token") || error.contains("API Token is required") {
        Some("backendErrors.reporterConfigApiTokenMissing")
    } else if error.contains("缺少 GeneratedHashKey")
        || error.contains("Device key is required")
    {
        Some("backendErrors.reporterConfigGeneratedHashKeyMissing")
    } else if error.contains("仍在退出") || error.contains("still stopping") {
        Some("backendErrors.reporterWorkerStopping")
    } else {
        None
    }
}

fn discord_start_error_code(error: &str) -> Option<&'static str> {
    if error.contains("缺少站点地址") || error.contains("Site URL is required") {
        Some("backendErrors.discordConfigBaseUrlMissing")
    } else if error.contains("缺少 Discord Application ID")
        || error.contains("Discord Application ID is required")
    {
        Some("backendErrors.discordConfigAppIdMissing")
    } else if error.contains("缺少 Discord 来源标识")
        || error.contains("Discord source ID is required")
    {
        Some("backendErrors.discordConfigSourceIdMissing")
    } else if error.contains("仍在退出") || error.contains("still stopping") {
        Some("backendErrors.discordWorkerStopping")
    } else {
        None
    }
}

fn with_dc_source_metadata(metadata: Option<Value>, dc_source: &str) -> Value {
    let mut map = match metadata {
        Some(Value::Object(map)) => map,
        _ => Map::new(),
    };
    if !dc_source.trim().is_empty() {
        map.insert(
            "dc_source".into(),
            Value::String(dc_source.trim().to_string()),
        );
    }
    Value::Object(map)
}
