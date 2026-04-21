use serde_json::Value;

use super::helpers::{
    build_activity_report_body, build_connectivity_probe_body, build_inspiration_asset_body,
    build_inspiration_entry_body, validate_image_data_url, ValidationError,
};
use crate::{
    http_client::{request_json, request_json_payload},
    models::{ActivityPayload, ApiResult, ClientConfig, InspirationEntryCreateInput},
};

pub async fn submit_activity_report(
    config: ClientConfig,
    payload: ActivityPayload,
) -> Result<ApiResult<Value>, String> {
    let body = build_activity_report_body(&config, payload);

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

pub async fn get_public_activity_feed(config: ClientConfig) -> Result<ApiResult<Value>, String> {
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

pub async fn list_inspiration_entries(
    config: ClientConfig,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<ApiResult<Value>, String> {
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

pub async fn probe_connectivity(config: ClientConfig) -> Result<ApiResult<Value>, String> {
    let body = build_connectivity_probe_body(&config);

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

pub async fn create_inspiration_entry(
    config: ClientConfig,
    input: InspirationEntryCreateInput,
) -> Result<ApiResult<Value>, String> {
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

    let body = build_inspiration_entry_body(input);

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

pub async fn upload_inspiration_asset(
    config: ClientConfig,
    image_data_url: String,
) -> Result<ApiResult<Value>, String> {
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

    let body = build_inspiration_asset_body(&config, image_data_url);

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
