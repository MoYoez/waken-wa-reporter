use std::{
    fs,
    io::Write,
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
        Ok(content) if content.trim().is_empty() => AppStatePayload::default(),
        Ok(content) => match serde_json::from_str(&content) {
            Ok(parsed) => parsed,
            Err(error) => {
                let backup = path.with_extension("json.corrupt");
                let _ = fs::copy(&path, &backup);
                eprintln!("客户端状态文件损坏，已备份到 {}：{error}", backup.display());
                AppStatePayload::default()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppStatePayload::default(),
        Err(error) => return Err(format!("读取状态文件失败：{error}")),
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
    atomic_write(&path, &content)?;
    Ok(())
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let mut file =
        fs::File::create(&tmp_path).map_err(|error| format!("创建临时状态文件失败：{error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("写入临时状态文件失败：{error}"))?;
    file.sync_all()
        .map_err(|error| format!("刷新临时状态文件失败：{error}"))?;
    drop(file);

    set_owner_only_permissions_if_supported(&tmp_path)?;

    fs::rename(&tmp_path, path).map_err(|error| {
        let _ = fs::remove_file(&tmp_path);
        format!("替换状态文件失败：{error}")
    })?;

    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions_if_supported(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("设置状态文件权限失败：{error}"))
}

#[cfg(not(unix))]
fn set_owner_only_permissions_if_supported(_path: &Path) -> Result<(), String> {
    Ok(())
}
