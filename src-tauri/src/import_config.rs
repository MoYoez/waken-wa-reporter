use base64::{engine::general_purpose, Engine as _};
use serde_json::{Map, Value};

use crate::models::ImportedIntegrationConfig;

fn parse_json_object(input: &str) -> Option<Map<String, Value>> {
    serde_json::from_str::<Value>(input)
        .ok()
        .and_then(|value| value.as_object().cloned())
}

fn decode_base64_json(input: &str) -> Option<Map<String, Value>> {
    let decoders = [
        general_purpose::STANDARD.decode(input),
        general_purpose::STANDARD_NO_PAD.decode(input),
        general_purpose::URL_SAFE.decode(input),
        general_purpose::URL_SAFE_NO_PAD.decode(input),
    ];

    decoders.into_iter().find_map(|result| {
        result
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| parse_json_object(&text))
    })
}

pub fn parse_import_payload(input: &str) -> Result<ImportedIntegrationConfig, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("请先粘贴 Base64 或 JSON 配置。".into());
    }

    let raw = parse_json_object(trimmed)
        .or_else(|| decode_base64_json(trimmed))
        .ok_or_else(|| "无法将输入解析为 Base64 配置或 JSON。".to_string())?;

    let token_block = raw.get("token").and_then(|value| value.as_object());
    let items = token_block
        .and_then(|token| token.get("items"))
        .and_then(|items| items.as_array());

    let first_item = items.and_then(|items| {
        items.iter().find_map(|item| {
            let object = item.as_object()?;
            object.get("token").and_then(Value::as_str)?;
            Some(object)
        })
    });

    let legacy_endpoint = raw
        .get("endpoint")
        .and_then(Value::as_str)
        .map(str::to_string);
    let legacy_api_key = raw
        .get("apiKey")
        .and_then(Value::as_str)
        .map(str::to_string);
    let legacy_token_name = raw
        .get("tokenName")
        .and_then(Value::as_str)
        .map(str::to_string);

    let report_endpoint = token_block
        .and_then(|token| token.get("reportEndpoint"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or(legacy_endpoint);

    Ok(ImportedIntegrationConfig {
        report_endpoint,
        token: first_item
            .and_then(|item| item.get("token"))
            .and_then(Value::as_str)
            .map(str::to_string),
        token_name: first_item
            .and_then(|item| item.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or(legacy_token_name),
        raw,
    }
    .with_legacy_token(legacy_api_key))
}

trait LegacyImportExt {
    fn with_legacy_token(self, legacy_token: Option<String>) -> Self;
}

impl LegacyImportExt for ImportedIntegrationConfig {
    fn with_legacy_token(mut self, legacy_token: Option<String>) -> Self {
        if self.token.is_none() {
            self.token = legacy_token;
        }
        self
    }
}
