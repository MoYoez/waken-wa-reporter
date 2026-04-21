use serde_json::json;
use tauri::AppHandle;

use crate::{
    import_config::parse_import_payload,
    models::{
        default_client_capabilities, ApiResult, AppStatePayload, ClientCapabilities,
        ExistingReporterConfig, ImportedIntegrationConfig,
    },
    reporter_config, state_store,
};

const MAX_IMPORT_INPUT_BYTES: usize = 512 * 1024;

pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state_store::save_app_state(&app, &payload)
}

pub fn restart_app(app: AppHandle) -> Result<(), String> {
    app.request_restart();
    Ok(())
}

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

pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    Ok(ApiResult::success(200, default_client_capabilities()))
}

pub fn discover_existing_reporter_config(
    app: AppHandle,
) -> Result<ApiResult<ExistingReporterConfig>, String> {
    let config = reporter_config::discover_existing_reporter_config(&app)?;
    Ok(ApiResult::success(200, config))
}
