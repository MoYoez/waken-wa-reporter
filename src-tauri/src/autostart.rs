#[cfg(target_os = "windows")]
mod windows_impl {
    use tauri::{AppHandle, Runtime};
    use winreg::enums::RegType::REG_BINARY;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::{RegKey, RegValue};

    const RUN_REGKEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
    const STARTUP_APPROVED_REGKEY: &str =
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    const STARTUP_APPROVED_ENABLED_VALUE: [u8; 12] =
        [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    fn app_name<R: Runtime>(app: &AppHandle<R>) -> String {
        app.package_info().name.clone()
    }

    fn run_command(args: &[&str]) -> Result<String, String> {
        let exe = std::env::current_exe()
            .map_err(|error| format!("无法获取可执行文件路径：{error}"))?;
        let joined_args = args.join(" ");
        if joined_args.is_empty() {
            Ok(exe.display().to_string())
        } else {
            Ok(format!("{} {}", exe.display(), joined_args))
        }
    }

    fn last_eight_bytes_all_zeros(bytes: &[u8]) -> Option<bool> {
        if bytes.len() < 8 {
            return None;
        }
        Some(bytes.iter().rev().take(8).all(|value| *value == 0u8))
    }

    pub fn enable<R: Runtime>(app: &AppHandle<R>, args: &[&str]) -> Result<(), String> {
        let name = app_name(app);
        let command = run_command(args)?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey_with_flags(RUN_REGKEY, KEY_SET_VALUE)
            .map_err(|error| format!("打开自启动注册表项失败：{error}"))?
            .set_value(&name, &command)
            .map_err(|error| format!("写入自启动注册表项失败：{error}"))?;

        if let Ok(reg) = hkcu.open_subkey_with_flags(STARTUP_APPROVED_REGKEY, KEY_SET_VALUE) {
            reg.set_raw_value(
                &name,
                &RegValue {
                    vtype: REG_BINARY,
                    bytes: STARTUP_APPROVED_ENABLED_VALUE.to_vec(),
                },
            )
            .map_err(|error| format!("写入任务管理器启动项覆盖值失败：{error}"))?;
        }

        Ok(())
    }

    pub fn disable<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        let name = app_name(app);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        match hkcu
            .open_subkey_with_flags(RUN_REGKEY, KEY_SET_VALUE)
            .map_err(|error| format!("打开自启动注册表项失败：{error}"))?
            .delete_value(&name)
        {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("删除自启动注册表项失败：{error}")),
        }
    }

    pub fn is_enabled<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
        let name = app_name(app);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_enabled = hkcu
            .open_subkey_with_flags(RUN_REGKEY, KEY_READ)
            .map_err(|error| format!("读取自启动注册表项失败：{error}"))?
            .get_value::<String, _>(&name)
            .is_ok();

        let approved_enabled = hkcu
            .open_subkey_with_flags(STARTUP_APPROVED_REGKEY, KEY_READ)
            .ok()
            .and_then(|reg| reg.get_raw_value(&name).ok())
            .and_then(|raw| last_eight_bytes_all_zeros(&raw.bytes))
            .unwrap_or(true);

        Ok(run_enabled && approved_enabled)
    }
}

#[cfg(target_os = "windows")]
pub fn enable(app: &tauri::AppHandle, args: &[&str]) -> Result<(), String> {
    windows_impl::enable(app, args)
}

#[cfg(target_os = "windows")]
pub fn disable(app: &tauri::AppHandle) -> Result<(), String> {
    windows_impl::disable(app)
}

#[cfg(target_os = "windows")]
pub fn is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    windows_impl::is_enabled(app)
}

#[cfg(not(target_os = "windows"))]
pub fn enable(app: &tauri::AppHandle, _args: &[&str]) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    app.autolaunch()
        .enable()
        .map_err(|error| format!("启用自启动失败：{error}"))
}

#[cfg(not(target_os = "windows"))]
pub fn disable(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    app.autolaunch()
        .disable()
        .map_err(|error| format!("禁用自启动失败：{error}"))
}

#[cfg(not(target_os = "windows"))]
pub fn is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;

    app.autolaunch()
        .is_enabled()
        .map_err(|error| format!("查询自启动状态失败：{error}"))
}

pub fn ensure_protected(app: &tauri::AppHandle, args: &[&str]) -> Result<(), String> {
    if is_enabled(app)? {
        return Ok(());
    }
    enable(app, args)
}
