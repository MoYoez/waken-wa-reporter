use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::backend_locale::BackendLocale;

use super::messages::{default_public_feed_failure, format_error, public_feed_status_error};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicActivityFeed {
    #[serde(default)]
    pub(super) active_statuses: Vec<PublicActivityItem>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicActivityItem {
    #[serde(default)]
    pub(super) device: Option<String>,
    #[serde(default)]
    pub(super) metadata: Option<Value>,
    #[serde(default)]
    pub(super) process_name: Option<String>,
    #[serde(default)]
    pub(super) process_title: Option<String>,
    #[serde(default)]
    pub(super) status_text: Option<String>,
    #[serde(default)]
    pub(super) started_at: Option<String>,
}

pub(super) fn fetch_public_activity_feed_blocking(
    client: &Client,
    base_url: &str,
    locale: BackendLocale,
) -> Result<PublicActivityFeed, String> {
    let url = format!("{}/api/activity?public=1", base_url.trim_end_matches('/'));
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .map_err(|error| {
            format_error(
                locale,
                "拉取公开活动失败",
                "Failed to fetch public activity",
                error,
            )
        })?;

    let status = response.status().as_u16();
    let text = response.text().unwrap_or_default();
    if status >= 400 {
        return Err(public_feed_status_error(locale, status, text.trim()));
    }

    let payload = if text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
    };

    let success = payload
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    if !success {
        return Err(payload
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .map(str::to_string)
            .unwrap_or_else(|| default_public_feed_failure(locale)));
    }

    let data = payload.get("data").cloned().unwrap_or(payload);
    serde_json::from_value::<PublicActivityFeed>(data).map_err(|error| {
        format_error(
            locale,
            "解析公开活动失败",
            "Failed to parse public activity",
            error,
        )
    })
}

pub(super) fn select_dc_source_activity<'a>(
    active_statuses: &'a [PublicActivityItem],
    target_dc_source: &str,
) -> Option<&'a PublicActivityItem> {
    let normalized_target = target_dc_source.trim();
    if normalized_target.is_empty() {
        return None;
    }

    active_statuses
        .iter()
        .find(|item| item_dc_source(item) == Some(normalized_target))
}

pub(super) fn item_dc_source(item: &PublicActivityItem) -> Option<&str> {
    item.metadata
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("dc_source"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
