use std::{
    env, fs,
    path::Path,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::json;
use serde_json::Value;

use super::{
    build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo,
    ProbeTextSpec,
};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland() {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11() {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11().or_else(|x11_error| {
        get_foreground_snapshot_wayland().map_err(|wayland_error| {
            format!("读取 Linux 前台窗口失败。X11：{x11_error}；Wayland：{wayland_error}")
        })
    })
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland_for_reporting(
            include_process_name,
            include_process_title,
        ) {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11_for_reporting(
                include_process_name,
                include_process_title,
            ) {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11_for_reporting(include_process_name, include_process_title).or_else(
        |x11_error| {
            get_foreground_snapshot_wayland_for_reporting(
                include_process_name,
                include_process_title,
            )
            .map_err(|wayland_error| {
                format!("读取 Linux 前台窗口失败。X11：{x11_error}；Wayland：{wayland_error}")
            })
        },
    )
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}\n{{playerName}}",
        ],
    )
    .map_err(|error| format!("调用 playerctl 失败：{error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(MediaInfo::default());
        }
        return Err(format!(
            "读取媒体信息失败：{}",
            stderr.trim().if_empty("playerctl 返回失败")
        ));
    }

    let mut lines = stdout.lines().map(str::trim);
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();
    let source_app_id = lines.next().unwrap_or_default().to_string();

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

fn get_foreground_snapshot_x11() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_x11_for_reporting(true, true)
}

fn get_foreground_snapshot_x11_for_reporting(
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

fn get_foreground_snapshot_wayland() -> Result<ForegroundSnapshot, String> {
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

fn get_foreground_snapshot_wayland_for_reporting(
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

fn run_command_trimmed<const N: usize>(program: &str, args: [&str; N]) -> Result<String, String> {
    let output = command_output_with_timeout(program, &args)
        .map_err(|error| format!("调用 {program} 失败：{error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("命令返回失败")
            .to_string());
    }

    Ok(stdout.lines().next().unwrap_or_default().trim().to_string())
}

fn command_output_with_timeout(program: &str, args: &[&str]) -> Result<Output, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "命令执行超时（>{}ms）",
                    COMMAND_TIMEOUT.as_millis()
                ));
            }
            Ok(None) => thread::sleep(COMMAND_POLL_STEP),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
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

fn has_env(key: &str) -> bool {
    env::var(key)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn linux_detail(error: &str, probe: &str) -> ProbeTextSpec {
    let lower = error.to_lowercase();

    if probe == "media" {
        if lower.contains("playerctl") {
            return localized_text(
                "platformSelfTest.detail.linuxMediaPlayerctlMissing",
                None,
                error,
            );
        }

        return localized_text(
            "platformSelfTest.detail.linuxMediaUnavailable",
            None,
            error,
        );
    }

    if lower.contains("focused window d-bus") || lower.contains("gdbus") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundGnomeSupportMissing",
            None,
            error,
        );
    }

    if lower.contains("kdotool") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundKdeSupportMissing",
            None,
            error,
        );
    }

    if lower.contains("xprop") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundXpropMissing",
            None,
            error,
        );
    }

    localized_text(
        "platformSelfTest.detail.linuxForegroundUnavailable",
        None,
        error,
    )
}

fn linux_guidance(error: &str, probe: &str) -> Vec<ProbeTextSpec> {
    let lower = error.to_lowercase();
    let mut guidance = Vec::new();

    if probe == "foreground" || lower.contains("wayland") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxX11",
            None,
            "X11 会话可直接通过 xprop 读取前台窗口。",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxGnomeFocusedWindow",
            None,
            "GNOME Wayland 可安装 Focused Window D-Bus 扩展，客户端会直接通过 gdbus 读取前台窗口。",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxKdeKdotool",
            None,
            "KDE Plasma Wayland 可安装 kdotool，客户端会直接读取活动窗口类名和标题。",
        ));
    }

    if lower.contains("xprop") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallXprop",
            None,
            "请安装 xprop（通常由 xorg-xprop / x11-utils 提供）。",
        ));
    }

    if lower.contains("focused window d-bus") || lower.contains("gdbus") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxGnomeFocusedWindow",
            None,
            "GNOME 请安装 Focused Window D-Bus 扩展，并确认系统存在 gdbus。",
        ));
    }

    if lower.contains("kdotool") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallKdotool",
            None,
            "KDE Plasma 请安装 kdotool。",
        ));
    }

    if probe == "media" || lower.contains("playerctl") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallPlayerctl",
            None,
            "请安装 playerctl，并确认播放器实现了 MPRIS。",
        ));
    }

    if guidance.is_empty() {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxCheckDesktopPermission",
            None,
            "请先确认当前桌面环境是否允许采集前台窗口/媒体信息。",
        ));
    }

    guidance
}

pub fn run_self_test() -> PlatformSelfTestResult {
    let foreground = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            true,
            localized_text(
                "platformSelfTest.summary.foregroundOk",
                None,
                "前台应用采集正常",
            ),
            localized_text(
                "platformSelfTest.detail.foregroundCurrent",
                Some(json!({ "processName": snapshot.process_name.clone() })),
                format!("当前前台应用：{}", snapshot.process_name),
            ),
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.foregroundFailed",
                None,
                "前台应用采集失败",
            ),
            linux_detail(&error, "foreground"),
            linux_guidance(&error, "foreground"),
        ),
    };

    let window_title = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            !snapshot.process_title.trim().is_empty(),
            if snapshot.process_title.trim().is_empty() {
                localized_text(
                    "platformSelfTest.summary.windowTitleEmpty",
                    None,
                    "窗口标题为空",
                )
            } else {
                localized_text(
                    "platformSelfTest.summary.windowTitleOk",
                    None,
                    "窗口标题采集正常",
                )
            },
            if snapshot.process_title.trim().is_empty() {
                localized_text(
                    "platformSelfTest.detail.windowTitleEmpty",
                    None,
                    "当前前台窗口没有可用标题。",
                )
            } else {
                localized_text(
                    "platformSelfTest.detail.windowTitleCurrent",
                    Some(json!({ "processTitle": snapshot.process_title.clone() })),
                    snapshot.process_title,
                )
            },
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "窗口标题采集失败",
            ),
            linux_detail(&error, "window"),
            linux_guidance(&error, "foreground"),
        ),
    };

    let media = match get_now_playing() {
        Ok(info) if !info.is_empty() => {
            make_probe(
                true,
                localized_text("platformSelfTest.summary.mediaOk", None, "媒体采集正常"),
                localized_text(
                    "platformSelfTest.detail.mediaCurrent",
                    Some(json!({ "mediaSummary": info.summary() })),
                    info.summary(),
                ),
                Vec::new(),
            )
        }
        Ok(_) => make_probe(
            true,
            localized_text(
                "platformSelfTest.summary.mediaNone",
                None,
                "当前没有播放中的媒体",
            ),
            localized_text(
                "platformSelfTest.detail.mediaNone",
                None,
                "系统当前没有可用的正在播放信息。",
            ),
            vec![localized_text(
                "platformSelfTest.guidance.playMediaThenRetry",
                None,
                "如需验证媒体采集，请先播放一段音频/视频后重试。",
            )],
        ),
        Err(error) => make_probe(
            false,
            localized_text("platformSelfTest.summary.mediaFailed", None, "媒体采集失败"),
            linux_detail(&error, "media"),
            linux_guidance(&error, "media"),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

trait EmptyFallback {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl EmptyFallback for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("当前平台不支持辅助功能权限申请。".into())
}
