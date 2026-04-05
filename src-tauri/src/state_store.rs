use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::models::AppStatePayload;

const APP_STATE_FILE_NAME: &str = "client-state.json";

fn app_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法获取应用配置目录：{error}"))?;
    Ok(dir.join(APP_STATE_FILE_NAME))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建配置目录失败：{error}"))?;
    }
    Ok(())
}

fn ensure_generated_hash_key(payload: &mut AppStatePayload) -> bool {
    if payload.config.generated_hash_key.trim().is_empty() {
        payload.config.generated_hash_key = format!("wwd-{}", Uuid::new_v4());
        return true;
    }
    false
}

pub fn load_app_state(app: &AppHandle) -> Result<AppStatePayload, String> {
    let path = app_state_path(app)?;
    let mut payload = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppStatePayload::default(),
    };

    if ensure_generated_hash_key(&mut payload) {
        save_app_state(app, &payload)?;
    }

    Ok(payload)
}

pub fn save_app_state(app: &AppHandle, payload: &AppStatePayload) -> Result<(), String> {
    let path = app_state_path(app)?;
    ensure_parent_dir(&path)?;
    let mut payload = payload.clone();
    let _ = ensure_generated_hash_key(&mut payload);
    let content = serde_json::to_string_pretty(&payload)
        .map_err(|error| format!("序列化状态失败：{error}"))?;
    fs::write(path, content).map_err(|error| format!("写入状态文件失败：{error}"))?;
    Ok(())
}
