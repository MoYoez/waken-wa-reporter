use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Map, Value};

use crate::{
    backend_locale::BackendLocale,
    http_client::build_blocking_client,
    models::{effective_device_name, ActivityPayload, ClientConfig},
    platform::{ForegroundSnapshot, MediaInfo},
};

use super::messages::{
    format_error, pending_approval_default, reporter_config_api_token_missing,
    reporter_config_base_url_missing, reporter_config_generated_hash_key_missing,
    server_failure_default, server_status_error,
};

pub(super) enum PostActivityResult {
    Success {
        media_cover_url: Option<String>,
        response_text: String,
    },
    PendingApproval {
        message: String,
        approval_url: Option<String>,
        response_text: String,
    },
}

pub(super) fn validate_reporter_config(
    config: &ClientConfig,
    locale: BackendLocale,
) -> Result<(), String> {
    if config.base_url.trim().is_empty() {
        return Err(reporter_config_base_url_missing(locale));
    }
    if config.api_token.trim().is_empty() {
        return Err(reporter_config_api_token_missing(locale));
    }
    if config.generated_hash_key.trim().is_empty() {
        return Err(reporter_config_generated_hash_key_missing(locale));
    }
    Ok(())
}

pub(super) fn config_is_ready(config: &ClientConfig) -> bool {
    validate_reporter_config(config, BackendLocale::ZhCn).is_ok()
}

pub(super) fn build_http_client(
    use_system_proxy: bool,
    locale: BackendLocale,
) -> Result<Client, String> {
    build_blocking_client(
        "waken-wa-tauri-reporter/0.1.0",
        Some(Duration::from_secs(15)),
        use_system_proxy,
        locale,
    )
}

pub(super) fn parse_reporter_metadata(
    input: &str,
    locale: BackendLocale,
) -> Result<Map<String, Value>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Map::new());
    }

    match serde_json::from_str::<Value>(trimmed) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(if locale.is_en() {
            "Reporter metadata must be a JSON object.".into()
        } else {
            "实时上报元数据必须是 JSON 对象。".into()
        }),
        Err(error) => Err(format_error(
            locale,
            "解析实时上报元数据失败",
            "Failed to parse reporter metadata",
            error,
        )),
    }
}

pub(super) fn build_payload(
    config: &ClientConfig,
    snapshot: &ForegroundSnapshot,
    media: &MediaInfo,
    mut metadata: Map<String, Value>,
) -> ActivityPayload {
    // Only include dc_source when Discord presence is enabled
    if config.discord_enabled && !config.discord_source_id.trim().is_empty() {
        metadata.insert(
            "dc_source".into(),
            Value::String(config.discord_source_id.trim().to_string()),
        );
    }

    if config.report_media {
        if let Some(media_map) = media.as_metadata_map() {
            metadata.insert("media".into(), Value::Object(media_map));
        }
    }

    if config.report_play_source
        && !media.source_app_id.trim().is_empty()
        && !metadata.contains_key("play_source")
    {
        metadata.insert(
            "play_source".into(),
            Value::String(media.source_app_id.trim().to_string()),
        );
    }

    ActivityPayload {
        generated_hash_key: config.generated_hash_key.trim().to_string(),
        process_name: if config.report_foreground_app || !snapshot.process_name.is_empty() {
            snapshot.process_name.clone()
        } else {
            String::new()
        },
        device: Some(effective_device_name(&config.device)),
        process_title: if config.report_window_title || !snapshot.process_title.is_empty() {
            Some(snapshot.process_title.clone()).filter(|value| !value.is_empty())
        } else {
            None
        },
        persist_minutes: None,
        battery_level: None,
        is_charging: None,
        device_type: Some(config.device_type.trim().to_string()).filter(|value| !value.is_empty()),
        push_mode: Some(config.push_mode.trim().to_string()).filter(|value| !value.is_empty()),
        metadata: (!metadata.is_empty()).then_some(Value::Object(metadata)),
    }
}

pub(super) fn post_activity_blocking(
    client: &Client,
    config: &ClientConfig,
    payload: &ActivityPayload,
    locale: BackendLocale,
) -> Result<PostActivityResult, String> {
    let body = serde_json::to_value(payload).map_err(|error| {
        format_error(
            locale,
            "序列化上报数据失败",
            "Failed to encode report payload",
            error,
        )
    })?;
    let url = format!("{}/api/activity", config.base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", config.api_token.trim()))
        .json(&body)
        .send()
        .map_err(|error| format_error(locale, "请求失败", "Request failed", error))?;

    let status = response.status().as_u16();
    let text = response.text().unwrap_or_default();
    if status >= 400 {
        return Err(server_status_error(locale, status, text.trim()));
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
            .map(str::to_string)
            .unwrap_or_else(|| pending_approval_default(locale));
        let approval_url = parsed
            .get("approvalUrl")
            .and_then(Value::as_str)
            .map(str::to_string);
        return Ok(PostActivityResult::PendingApproval {
            message,
            approval_url,
            response_text: text,
        });
    }

    if !success {
        return Err(parsed
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| parsed.get("message").and_then(Value::as_str))
            .map(str::to_string)
            .unwrap_or_else(|| server_failure_default(locale)));
    }

    Ok(PostActivityResult::Success {
        media_cover_url: parsed
            .get("data")
            .and_then(|d| d.get("metadata"))
            .and_then(|m| m.get("media"))
            .and_then(|media| media.get("coverUrl"))
            .and_then(Value::as_str)
            .map(str::to_string),
        response_text: text,
    })
}
