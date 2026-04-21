use std::{fs, path::Path};

use crate::platform::ForegroundSnapshot;

use super::command::{command_output_with_timeout, EmptyFallback};

pub(super) fn get_foreground_snapshot_x11() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_x11_for_reporting(true, true)
}

pub(super) fn get_foreground_snapshot_x11_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let window_id = get_active_window_id_x11()?;
    let detail_stdout =
        read_x11_window_detail(&window_id, include_process_name, include_process_title)?;

    let process_title = if include_process_title {
        parse_window_title(&detail_stdout).unwrap_or_default()
    } else {
        String::new()
    };

    let process_name = if include_process_name {
        let wm_class = parse_wm_class(&detail_stdout).unwrap_or_default();
        parse_window_pid(&detail_stdout)
            .and_then(read_process_name_from_pid)
            .or_else(|| {
                if wm_class.trim().is_empty() {
                    None
                } else {
                    Some(wm_class)
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn get_active_window_id_x11() -> Result<String, String> {
    let active_output = command_output_with_timeout("xprop", &["-root", "_NET_ACTIVE_WINDOW"])
        .map_err(|error| format!("调用 xprop 失败：{error}"))?;

    if !active_output.status.success() {
        let stderr = String::from_utf8_lossy(&active_output.stderr);
        return Err(format!(
            "读取活动窗口失败：{}",
            stderr.trim().if_empty("xprop 返回失败")
        ));
    }

    let active_stdout = String::from_utf8_lossy(&active_output.stdout);
    let window_id = parse_active_window_id(&active_stdout)
        .ok_or_else(|| "无法解析 _NET_ACTIVE_WINDOW。".to_string())?;

    if window_id == "0x0" {
        return Err("当前没有活动窗口。".into());
    }

    Ok(window_id)
}

fn read_x11_window_detail(
    window_id: &str,
    include_process_name: bool,
    include_process_title: bool,
) -> Result<String, String> {
    let mut args = vec!["-id", window_id];
    if include_process_name {
        args.push("WM_CLASS");
        args.push("_NET_WM_PID");
    }
    if include_process_title {
        args.push("_NET_WM_NAME");
        args.push("WM_NAME");
    }

    let detail_output = command_output_with_timeout("xprop", &args)
        .map_err(|error| format!("调用 xprop 读取窗口详情失败：{error}"))?;

    if !detail_output.status.success() {
        let stderr = String::from_utf8_lossy(&detail_output.stderr);
        return Err(format!(
            "读取窗口详情失败：{}",
            stderr.trim().if_empty("xprop 返回失败")
        ));
    }

    Ok(String::from_utf8_lossy(&detail_output.stdout).to_string())
}

fn parse_active_window_id(stdout: &str) -> Option<String> {
    stdout
        .split('#')
        .nth(1)
        .map(str::trim)
        .and_then(|value| value.split_whitespace().next())
        .map(str::to_string)
}

fn parse_wm_class(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if !line.starts_with("WM_CLASS") {
            continue;
        }
        let values = extract_quoted_values(line);
        if values.len() >= 2 {
            return Some(values[1].clone());
        }
        if let Some(value) = values.first() {
            return Some(value.clone());
        }
    }
    None
}

fn parse_window_title(stdout: &str) -> Option<String> {
    for key in ["_NET_WM_NAME", "WM_NAME"] {
        for line in stdout.lines() {
            if !line.starts_with(key) {
                continue;
            }
            let values = extract_quoted_values(line);
            if let Some(value) = values.first() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn parse_window_pid(stdout: &str) -> Option<u32> {
    for line in stdout.lines() {
        if !line.starts_with("_NET_WM_PID") {
            continue;
        }
        let raw = line.split('=').nth(1)?.trim();
        if let Ok(pid) = raw.parse::<u32>() {
            return Some(pid);
        }
    }
    None
}

fn extract_quoted_values(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start = None;

    for (idx, ch) in line.char_indices() {
        if ch == '"' {
            match start {
                Some(begin) => {
                    values.push(line[begin..idx].to_string());
                    start = None;
                }
                None => start = Some(idx + 1),
            }
        }
    }

    values
}

fn read_process_name_from_pid(pid: u32) -> Option<String> {
    let comm_path = Path::new("/proc").join(pid.to_string()).join("comm");
    let name = fs::read_to_string(comm_path).ok()?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
