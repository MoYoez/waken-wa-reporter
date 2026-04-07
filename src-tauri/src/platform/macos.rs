use std::ffi::{c_char, CStr};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);
const NOWPLAYING_CLI: &str = "nowplaying-cli";

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
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

enum NowPlayingCliError {
    NotFound { path: String },
    TimedOut,
    Failed(String),
}

impl NowPlayingCliError {
    fn into_user_message(self) -> String {
        match self {
            Self::NotFound { path } => {
                format!("调用 nowplaying-cli 失败：未在全局环境中找到可执行文件。PATH={path}")
            }
            Self::TimedOut => "调用 nowplaying-cli 超时（>1500ms）。".into(),
            Self::Failed(detail) => format!("nowplaying-cli 返回失败：{detail}"),
        }
    }
}

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, NowPlayingCliError> {
    let output = command_output_with_timeout(NOWPLAYING_CLI, &["get", "title", "artist", "album"])
        .map_err(|error| match error {
            CommandError::NotFound => NowPlayingCliError::NotFound {
                path: std::env::var("PATH").unwrap_or_default(),
            },
            CommandError::TimedOut => NowPlayingCliError::TimedOut,
            CommandError::Other(detail) => NowPlayingCliError::Failed(detail),
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
    let mut lines = stdout.lines().map(str::trim);
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();

    let normalize = |value: String| {
        if value.eq_ignore_ascii_case("null") {
            String::new()
        } else {
            value
        }
    };

    Ok(MediaInfo {
        title: normalize(title),
        artist: normalize(artist),
        album: normalize(album),
        source_app_id: NOWPLAYING_CLI.into(),
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

    Ok(ForegroundSnapshot {
        process_name,
        process_title: String::new(),
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
        guidance.push("当前版本暂未在 macOS 上实现窗口标题采集。".into());
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

    let window_title = make_probe(
        false,
        "窗口标题暂未支持",
        "当前版本已移除 osascript，macOS 上暂未实现窗口标题采集。".to_string(),
        macos_guidance("", "window"),
    );

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
