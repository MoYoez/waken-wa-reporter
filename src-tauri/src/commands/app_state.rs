use serde::Serialize;
use serde_json::{json, Value};
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
const MIN_POLL_INTERVAL_MS: u64 = 1_000;
const MIN_HEARTBEAT_INTERVAL_MS: u64 = 0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigValidationIssue {
    field: &'static str,
    path: &'static str,
    reason: &'static str,
    received: Value,
    expected: String,
    min: u64,
    suggested_value: u64,
}

pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

pub fn save_app_state(
    app: AppHandle,
    payload: Value,
) -> Result<ApiResult<AppStatePayload>, String> {
    let validation_issues = validate_reporter_timing_payload(&payload);
    if !validation_issues.is_empty() {
        return Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.settingsConfigInvalid".to_string()),
            "设置参数错误。",
            Some(json!({ "count": validation_issues.len() })),
            Some(json!({
                "issues": validation_issues,
                "values": suggested_reporter_timing_values(&payload),
            })),
        ));
    }

    let parsed_payload: AppStatePayload = match serde_json::from_value(payload) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Ok(ApiResult::failure_localized(
                400,
                Some("backendErrors.settingsPayloadInvalid".to_string()),
                format!("设置数据格式错误：{error}"),
                None,
                Some(json!({ "error": error.to_string() })),
            ));
        }
    };

    state_store::save_app_state(&app, &parsed_payload)?;
    Ok(ApiResult::success(200, parsed_payload))
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

fn validate_reporter_timing_payload(payload: &Value) -> Vec<ConfigValidationIssue> {
    let config = payload.get("config");
    let mut issues = Vec::new();

    if let Some(issue) = validate_timing_field(
        config,
        "pollIntervalMs",
        "config.pollIntervalMs",
        MIN_POLL_INTERVAL_MS,
        crate::models::default_poll_interval_ms(),
    ) {
        issues.push(issue);
    }

    if let Some(issue) = validate_timing_field(
        config,
        "heartbeatIntervalMs",
        "config.heartbeatIntervalMs",
        MIN_HEARTBEAT_INTERVAL_MS,
        crate::models::default_heartbeat_interval_ms(),
    ) {
        issues.push(issue);
    }

    issues
}

fn validate_timing_field(
    config: Option<&Value>,
    field: &'static str,
    path: &'static str,
    min: u64,
    default_value: u64,
) -> Option<ConfigValidationIssue> {
    let value = config.and_then(|config| config.get(field))?;

    let number = match value {
        Value::Number(number) => number,
        Value::Null => {
            return Some(timing_issue(
                field,
                path,
                "empty",
                value.clone(),
                min,
                default_value,
            ));
        }
        _ => {
            return Some(timing_issue(
                field,
                path,
                "notInteger",
                value.clone(),
                min,
                default_value,
            ));
        }
    };

    let Some(parsed) = number.as_u64() else {
        return Some(timing_issue(
            field,
            path,
            "notInteger",
            value.clone(),
            min,
            default_value,
        ));
    };

    if parsed < min {
        return Some(timing_issue(
            field,
            path,
            "tooSmall",
            value.clone(),
            min,
            min,
        ));
    }

    None
}

fn timing_issue(
    field: &'static str,
    path: &'static str,
    reason: &'static str,
    received: Value,
    min: u64,
    suggested_value: u64,
) -> ConfigValidationIssue {
    ConfigValidationIssue {
        field,
        path,
        reason,
        received,
        expected: format!("integer >= {min}"),
        min,
        suggested_value,
    }
}

fn suggested_reporter_timing_values(payload: &Value) -> Value {
    let config = payload.get("config");
    json!({
        "pollIntervalMs": suggested_timing_value(
            config.and_then(|config| config.get("pollIntervalMs")),
            MIN_POLL_INTERVAL_MS,
            crate::models::default_poll_interval_ms(),
        ),
        "heartbeatIntervalMs": suggested_timing_value(
            config.and_then(|config| config.get("heartbeatIntervalMs")),
            MIN_HEARTBEAT_INTERVAL_MS,
            crate::models::default_heartbeat_interval_ms(),
        ),
    })
}

fn suggested_timing_value(value: Option<&Value>, min: u64, default_value: u64) -> u64 {
    let Some(Value::Number(number)) = value else {
        return default_value;
    };

    number
        .as_u64()
        .map(|value| value.max(min))
        .unwrap_or(default_value)
}
