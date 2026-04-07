use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::{
    sync::Mutex,
    time::Duration,
};

use crate::models::ApiResult;

static ASYNC_CLIENT_SYSTEM_PROXY: Mutex<Option<reqwest::Client>> = Mutex::new(None);
static ASYNC_CLIENT_DIRECT: Mutex<Option<reqwest::Client>> = Mutex::new(None);

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').trim().to_string()
}

fn extract_message(payload: &Value) -> Option<String> {
    payload
        .get("error")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

pub async fn request_json(
    base_url: &str,
    path: &str,
    token: Option<&str>,
    use_system_proxy: bool,
    method: reqwest::Method,
    body: Option<Value>,
) -> ApiResult<Value> {
    let client = match get_or_create_async_client(
        "waken-wa-tauri-client/0.1.0",
        Some(Duration::from_secs(15)),
        use_system_proxy,
    ) {
        Ok(client) => client,
        Err(error) => return ApiResult::failure(0, error, None),
    };

    let url = format!("{}{}", normalize_base_url(base_url), path);
    let mut request = client
        .request(method, url)
        .header(CONTENT_TYPE, "application/json");

    if let Some(token) = token.filter(|token| !token.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => return ApiResult::failure(0, format!("请求失败：{error}"), None),
    };

    let status = response.status().as_u16();
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return ApiResult::failure(status, format!("读取响应失败：{error}"), None),
    };

    let payload = if text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
    };

    let payload_success = payload
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(status < 400);

    if status >= 400 || !payload_success {
        return ApiResult::failure(
            status,
            extract_message(&payload).unwrap_or_else(|| "请求失败".into()),
            Some(payload),
        );
    }

    let data = payload.get("data").cloned().unwrap_or(payload);
    ApiResult::success(status, data)
}

fn get_or_create_async_client(
    user_agent: &str,
    timeout: Option<Duration>,
    use_system_proxy: bool,
) -> Result<reqwest::Client, String> {
    let cache = if use_system_proxy {
        &ASYNC_CLIENT_SYSTEM_PROXY
    } else {
        &ASYNC_CLIENT_DIRECT
    };

    let mut guard = cache.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    let client = build_async_client(user_agent, timeout, use_system_proxy)?;
    *guard = Some(client.clone());
    Ok(client)
}

pub fn build_async_client(
    user_agent: &str,
    timeout: Option<Duration>,
    use_system_proxy: bool,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().user_agent(user_agent);

    if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
    }

    if !use_system_proxy {
        builder = builder.no_proxy();
    }

    builder
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败：{error}"))
}

pub fn build_blocking_client(
    user_agent: &str,
    timeout: Option<Duration>,
    use_system_proxy: bool,
) -> Result<reqwest::blocking::Client, String> {
    let mut builder = reqwest::blocking::Client::builder().user_agent(user_agent);

    if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
    }

    if !use_system_proxy {
        builder = builder.no_proxy();
    }

    builder
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败：{error}"))
}
