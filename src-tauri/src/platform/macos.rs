use std::ffi::{c_char, CStr};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);

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

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, String> {
    let candidates = [
        "nowplaying-cli",
        "/usr/local/bin/nowplaying-cli",
        "/opt/homebrew/bin/nowplaying-cli",
    ];
    let mut last_error = None;

    for candidate in candidates {
        if candidate.contains('/') && !Path::new(candidate).exists() {
            continue;
        }

        match command_output_with_timeout(candidate, &["get", "title", "artist", "album"]) {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!(
                        "nowplaying-cli 返回失败（{}）：{}",
                        candidate,
                        stderr.trim()
                    ));
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

                return Ok(MediaInfo {
                    title: normalize(title),
                    artist: normalize(artist),
                    album: normalize(album),
                    source_app_id: "nowplaying-cli".into(),
                });
            }
            Err(error) => {
                last_error = Some(format!("{}: {}", candidate, error));
            }
        }
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    Err(format!(
        "调用 nowplaying-cli 失败：未在可用路径中找到可执行文件。已尝试：{}。PATH={}. 最后一次错误：{}",
        candidates.join(", "),
        current_path,
        last_error.unwrap_or_else(|| "未知错误".to_string())
    ))
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
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
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
    let media = get_now_playing_via_nowplaying_cli()?;
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
        guidance.push("确认当前 macOS 上确实有正在播放的媒体内容。".into());
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
