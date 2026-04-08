use std::ffi::{c_char, CStr};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);
const NOWPLAYING_CLI: &str = "nowplaying-cli";
const NOWPLAYING_CLI_FALLBACK_PATHS: [&str; 2] = [
    "/opt/homebrew/bin/nowplaying-cli",
    "/usr/local/bin/nowplaying-cli",
];

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
    fn waken_frontmost_window_title() -> *mut c_char;
    fn waken_accessibility_is_trusted() -> bool;
    fn waken_request_accessibility_permission() -> bool;
    fn waken_string_free(value: *mut c_char);
}

fn read_bridge_string(fetch: unsafe extern "C" fn() -> *mut c_char) -> Option<String> {
    let ptr = unsafe { fetch() };
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(value)
}

pub fn accessibility_permission_granted() -> bool {
    unsafe { waken_accessibility_is_trusted() }
}

fn request_accessibility_permission_via_bridge() -> bool {
    unsafe { waken_request_accessibility_permission() }
}

enum NowPlayingCliError {
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl NowPlayingCliError {
    fn into_user_message(self) -> String {
        match self {
            Self::NotFound { path, attempted } => {
                format!(
                    "调用 nowplaying-cli 失败：未在全局环境或 Homebrew 常见路径中找到可执行文件。已尝试：{}。PATH={path}",
                    attempted.join(", ")
                )
            }
            Self::TimedOut => "调用 nowplaying-cli 超时（>1500ms）。".into(),
            Self::Failed(detail) => format!("nowplaying-cli 返回失败：{detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoTitle")]
    title: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoArtist")]
    artist: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoAlbum")]
    album: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoClientBundleIdentifier")]
    client_bundle_identifier: Option<String>,
}

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, NowPlayingCliError> {
    let attempted = std::iter::once(NOWPLAYING_CLI)
        .chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let output = {
        let mut resolved = None;
        for candidate in
            std::iter::once(NOWPLAYING_CLI).chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        {
            match command_output_with_timeout(candidate, &["get-raw"]) {
                Ok(output) => {
                    resolved = Some(output);
                    break;
                }
                Err(CommandError::NotFound) => {}
                Err(CommandError::TimedOut) => return Err(NowPlayingCliError::TimedOut),
                Err(CommandError::Other(detail)) => return Err(NowPlayingCliError::Failed(detail)),
            }
        }
        resolved
    }
    .ok_or_else(|| NowPlayingCliError::NotFound {
        path: std::env::var("PATH").unwrap_or_default(),
        attempted,
    })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
            || combined.contains("null")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("未知错误");
        return Err(NowPlayingCliError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalize = |value: String| {
        if value.eq_ignore_ascii_case("null") {
            String::new()
        } else {
            value.trim().to_string()
        }
    };

    let raw: RawNowPlayingInfo = serde_json::from_str(&stdout)
        .map_err(|error| NowPlayingCliError::Failed(format!("解析 get-raw 输出失败：{error}")))?;

    let title = raw.title.map(normalize).unwrap_or_default();
    let artist = raw.artist.map(normalize).unwrap_or_default();
    let album = raw.album.map(normalize).unwrap_or_default();
    let source_app_id = raw
        .client_bundle_identifier
        .map(normalize)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| NOWPLAYING_CLI.to_string());

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
    })
}

enum CommandError {
    NotFound,
    TimedOut,
    Other(String),
}

fn command_output_with_timeout(program: &str, args: &[&str]) -> Result<Output, CommandError> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => CommandError::NotFound,
            _ => CommandError::Other(error.to_string()),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| CommandError::Other(error.to_string()))
            }
            Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::TimedOut);
            }
            Ok(None) => thread::sleep(COMMAND_POLL_STEP),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::Other(error.to_string()));
            }
        }
    }
}

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let process_name = read_bridge_string(waken_frontmost_app_name)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "读取 macOS 前台应用失败。".to_string())?;
    let process_title = read_bridge_string(waken_frontmost_window_title).unwrap_or_default();

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    if include_process_name {
        let mut snapshot = get_foreground_snapshot()?;
        if !include_process_title {
            snapshot.process_title.clear();
        }
        return Ok(snapshot);
    }

    let process_title = if include_process_title {
        read_bridge_string(waken_frontmost_window_title).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name: String::new(),
        process_title,
    })
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let media = match get_now_playing_via_nowplaying_cli() {
        Ok(media) => media,
        Err(NowPlayingCliError::TimedOut) => return Ok(MediaInfo::default()),
        Err(error) => return Err(error.into_user_message()),
    };
    if media.is_empty() {
        return Ok(MediaInfo::default());
    }
    Ok(media)
}

fn macos_guidance(error: &str, probe: &str) -> Vec<String> {
    let lower = error.to_lowercase();
    let mut guidance = Vec::new();

    if probe == "foreground" {
        guidance.push("当前版本的 macOS 前台应用采集只走原生桥接，不再使用 osascript。".into());
        guidance.push("如果仍然失败，请检查系统版本或是否存在窗口列表访问限制。".into());
    }

    if probe == "window" {
        if accessibility_permission_granted() {
            guidance.push(
                "已检测到“辅助功能”权限；如果仍然没有标题，通常是当前应用本身未暴露稳定标题。"
                    .into(),
            );
        } else {
            guidance.push("macOS 窗口标题采集依赖“辅助功能”授权。".into());
            guidance.push("可以在设置页点“申请辅助功能权限”，或前往“系统设置 -> 隐私与安全性 -> 辅助功能”手动开启。".into());
        }
        guidance.push("部分应用即使已授权，也可能不会返回稳定的窗口标题。".into());
    }

    if probe == "media" || lower.contains("nowplaying-cli") {
        guidance.push("请先安装 nowplaying-cli：`brew install nowplaying-cli`。".into());
        guidance
            .push("如果当前没有正在播放的媒体，客户端现在会直接返回空结果，不再记为失败。".into());
    }

    if guidance.is_empty() {
        guidance.push("如果这是权限问题，请先检查 macOS 的“辅助功能”和“自动化”授权。".into());
    }

    guidance
}

pub fn run_self_test() -> PlatformSelfTestResult {
    let foreground = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            true,
            "前台应用采集正常",
            format!("当前前台应用：{}", snapshot.process_name),
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            "前台应用采集失败",
            error.clone(),
            macos_guidance(&error, "foreground"),
        ),
    };

    let window_title = match get_foreground_snapshot_for_reporting(false, true) {
        Ok(snapshot) => make_probe(
            !snapshot.process_title.trim().is_empty(),
            if snapshot.process_title.trim().is_empty() {
                "窗口标题为空"
            } else {
                "窗口标题采集正常"
            },
            if snapshot.process_title.trim().is_empty() {
                if accessibility_permission_granted() {
                    "当前前台窗口没有可用标题。".to_string()
                } else {
                    "当前前台窗口没有可用标题，且尚未授予辅助功能权限。".to_string()
                }
            } else {
                snapshot.process_title
            },
            macos_guidance("", "window"),
        ),
        Err(error) => make_probe(
            false,
            "窗口标题采集失败",
            error.clone(),
            macos_guidance(&error, "window"),
        ),
    };

    let media = match get_now_playing() {
        Ok(info) if !info.is_empty() => {
            make_probe(true, "媒体采集正常", info.summary(), Vec::new())
        }
        Ok(_) => make_probe(
            true,
            "当前没有播放中的媒体",
            "系统当前没有可用的正在播放信息。".to_string(),
            vec!["如果你正在测试媒体采集，请先播放一段音乐后再运行自检。".into()],
        ),
        Err(error) => make_probe(
            false,
            "媒体采集失败",
            error.clone(),
            macos_guidance(&error, "media"),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Ok(request_accessibility_permission_via_bridge())
}
