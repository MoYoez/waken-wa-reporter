use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::models::{ClientConfig, ExistingReporterConfig};

#[derive(serde::Deserialize)]
struct ReporterConfigFile {
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    api_token: String,
    #[serde(default)]
    device_name: String,
    #[serde(default)]
    generated_hash_key: String,
    #[serde(default)]
    poll_interval_ms: Option<u64>,
    #[serde(default)]
    heartbeat_interval_ms: Option<u64>,
    #[serde(default)]
    metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

fn reporter_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法获取应用配置目录：{error}"))?;

    let base = dir
        .parent()
        .ok_or_else(|| "无法定位用户配置目录。".to_string())?;

    Ok(base.join("waken-wa").join("config.json"))
}

pub fn discover_existing_reporter_config(
    app: &AppHandle,
) -> Result<ExistingReporterConfig, String> {
    let path = reporter_config_path(app)?;
    if !path.exists() {
        return Ok(ExistingReporterConfig {
            found: false,
            path: Some(path.display().to_string()),
            config: None,
        });
    }

    let content =
        fs::read_to_string(&path).map_err(|error| format!("读取 reporter 配置失败：{error}"))?;
    let parsed: ReporterConfigFile = serde_json::from_str(&content)
        .map_err(|error| format!("解析 reporter 配置失败：{error}"))?;

    let metadata_json = parsed
        .metadata
        .map(serde_json::Value::Object)
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(crate::models::default_reporter_metadata_json);

    Ok(ExistingReporterConfig {
        found: true,
        path: Some(path.display().to_string()),
        config: Some(ClientConfig {
            base_url: parsed.base_url.trim_end_matches('/').to_string(),
            api_token: parsed.api_token,
            generated_hash_key: if parsed.generated_hash_key.trim().is_empty() {
                format!("wwd-{}", Uuid::new_v4())
            } else {
                parsed.generated_hash_key
            },
            device: if parsed.device_name.trim().is_empty() {
                crate::models::default_device_name()
            } else {
                parsed.device_name
            },
            device_type: crate::models::default_device_type(),
            push_mode: crate::models::default_push_mode(),
            poll_interval_ms: parsed
                .poll_interval_ms
                .filter(|value| *value >= 1)
                .unwrap_or_else(crate::models::default_poll_interval_ms),
            heartbeat_interval_ms: parsed
                .heartbeat_interval_ms
                .unwrap_or_else(crate::models::default_heartbeat_interval_ms),
            reporter_metadata_json: metadata_json,
            reporter_enabled: false,
        }),
    })
}
