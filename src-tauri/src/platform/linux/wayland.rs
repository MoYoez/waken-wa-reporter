use serde_json::Value;

use crate::platform::ForegroundSnapshot;

use super::command::{command_output_with_timeout, run_command_trimmed, EmptyFallback};

pub(super) fn get_foreground_snapshot_wayland() -> Result<ForegroundSnapshot, String> {
    let mut errors = Vec::new();

    match get_foreground_snapshot_gnome_focused_window_dbus() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("GNOME Focused Window D-Bus：{error}")),
    }

    match get_foreground_snapshot_kde_kdotool() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("KDE kdotool：{error}")),
    }

    Err(format!("Wayland 前台窗口采集失败。{}", errors.join("；")))
}

pub(super) fn get_foreground_snapshot_wayland_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let mut errors = Vec::new();

    match get_foreground_snapshot_gnome_focused_window_dbus() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("GNOME Focused Window D-Bus：{error}")),
    }

    match get_foreground_snapshot_kde_kdotool_for_reporting(
        include_process_name,
        include_process_title,
    ) {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("KDE kdotool：{error}")),
    }

    Err(format!("Wayland 前台窗口采集失败。{}", errors.join("；")))
}

fn get_foreground_snapshot_gnome_focused_window_dbus() -> Result<ForegroundSnapshot, String> {
    let output = command_output_with_timeout(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/shell/extensions/FocusedWindow",
            "--method",
            "org.gnome.shell.extensions.FocusedWindow.Get",
        ],
    )
    .map_err(|error| format!("调用 gdbus 失败：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "调用 Focused Window D-Bus 失败：{}",
            stderr.trim().if_empty("gdbus 返回失败")
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_payload = parse_gdbus_string_tuple(&stdout)
        .ok_or_else(|| "无法解析 Focused Window D-Bus 返回值。".to_string())?;
    let payload: Value = serde_json::from_str(&json_payload)
        .map_err(|error| format!("解析 Focused Window D-Bus JSON 失败：{error}"))?;

    let process_name = [
        value_as_trimmed_string(payload.get("wm_class_instance")),
        value_as_trimmed_string(payload.get("wm_class")),
        value_as_trimmed_string(payload.get("app_id")),
    ]
    .into_iter()
    .flatten()
    .next()
    .ok_or_else(|| "Focused Window D-Bus 未返回可用的窗口类名。".to_string())?;

    let process_title = value_as_trimmed_string(payload.get("title")).unwrap_or_default();

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn get_foreground_snapshot_kde_kdotool() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_kde_kdotool_for_reporting(true, true)
}

fn get_foreground_snapshot_kde_kdotool_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let window_id = run_command_trimmed("kdotool", ["getactivewindow"])
        .map_err(|error| format!("读取活动窗口失败：{error}"))?;
    if window_id == "0" {
        return Err("当前没有活动窗口。".into());
    }

    let process_name = if include_process_name {
        let value = run_command_trimmed("kdotool", ["getwindowclassname", &window_id])
            .map_err(|error| format!("读取窗口类名失败：{error}"))?;
        if value.is_empty() {
            return Err("kdotool 未返回窗口类名。".into());
        }
        value
    } else {
        String::new()
    };

    let process_title = if include_process_title {
        run_command_trimmed("kdotool", ["getwindowname", &window_id]).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn parse_gdbus_string_tuple(stdout: &str) -> Option<String> {
    let start = stdout.find('\'')?;
    let mut escaped = false;
    let mut value = String::new();

    for ch in stdout[start + 1..].chars() {
        if escaped {
            value.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '\'' => return Some(value),
            other => value.push(other),
        }
    }

    None
}

fn value_as_trimmed_string(value: Option<&Value>) -> Option<String> {
    let raw = value?.as_str()?.trim();
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}
