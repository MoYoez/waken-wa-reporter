use serde_json::{json, Map, Value};

use crate::models::{
    effective_device_name, ActivityPayload, ClientConfig, InspirationEntryCreateInput,
};

const MAX_IMAGE_DATA_URL_BYTES: usize = 7 * 1024 * 1024;

pub(super) struct ValidationError {
    pub(super) code: &'static str,
    pub(super) message: String,
    pub(super) params: Option<Value>,
}

pub(super) fn build_activity_report_body(config: &ClientConfig, payload: ActivityPayload) -> Value {
    let generated_hash_key = if payload.generated_hash_key.trim().is_empty() {
        config.generated_hash_key.trim().to_string()
    } else {
        payload.generated_hash_key.trim().to_string()
    };

    let resolved_device = payload
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| effective_device_name(&config.device));

    json!({
        "generatedHashKey": generated_hash_key,
        "process_name": payload.process_name.trim(),
        "device": resolved_device,
        "process_title": normalize_optional_trimmed(payload.process_title),
        "persist_minutes": payload.persist_minutes,
        "battery_level": payload.battery_level,
        "is_charging": payload.is_charging,
        "device_type": normalize_optional_trimmed(payload.device_type),
        "push_mode": normalize_optional_trimmed(payload.push_mode),
        "metadata": with_dc_source_metadata(payload.metadata, &config.discord_source_id),
    })
}

pub(super) fn build_connectivity_probe_body(config: &ClientConfig) -> Value {
    json!({
        "generatedHashKey": config.generated_hash_key.trim(),
        "device": effective_device_name(&config.device),
        "deviceType": config.device_type.trim(),
    })
}

pub(super) fn build_inspiration_entry_body(input: InspirationEntryCreateInput) -> Value {
    json!({
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
    })
}

pub(super) fn build_inspiration_asset_body(config: &ClientConfig, image_data_url: String) -> Value {
    json!({
        "imageDataUrl": image_data_url,
        "generatedHashKey": config.generated_hash_key.trim(),
    })
}

pub(super) fn validate_image_data_url(value: Option<&str>) -> Result<(), ValidationError> {
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

pub(super) fn reporter_start_error_code(error: &str) -> Option<&'static str> {
    if error.contains("缺少站点地址") || error.contains("Site URL is required") {
        Some("backendErrors.reporterConfigBaseUrlMissing")
    } else if error.contains("缺少 API Token") || error.contains("API Token is required") {
        Some("backendErrors.reporterConfigApiTokenMissing")
    } else if error.contains("缺少 GeneratedHashKey") || error.contains("Device key is required")
    {
        Some("backendErrors.reporterConfigGeneratedHashKeyMissing")
    } else if error.contains("仍在退出") || error.contains("still stopping") {
        Some("backendErrors.reporterWorkerStopping")
    } else {
        None
    }
}

pub(super) fn discord_start_error_code(error: &str) -> Option<&'static str> {
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

fn normalize_optional_trimmed<T>(value: Option<T>) -> Option<String>
where
    T: AsRef<str>,
{
    value
        .map(|value| value.as_ref().trim().to_string())
        .filter(|value| !value.is_empty())
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
