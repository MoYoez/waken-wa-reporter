use std::{path::Path, process::Command};

use serde_json::Value;

use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

#[cfg(target_os = "windows")]
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    core::{HRESULT, PWSTR},
    Win32::{
        Foundation::{CloseHandle, MAX_PATH},
        System::{
            Com::{CoInitializeEx, COINIT_MULTITHREADED},
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    },
};

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err("读取前台窗口失败：GetForegroundWindow 返回空句柄。".into());
    }

    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    let process_title = if title_len <= 0 {
        String::new()
    } else {
        let mut buffer = vec![0u16; title_len as usize + 1];
        let written = unsafe { GetWindowTextW(hwnd, &mut buffer) };
        String::from_utf16_lossy(&buffer[..written as usize])
    };

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if pid == 0 {
        return Err("读取前台窗口失败：未能解析前台进程 ID。".into());
    }

    let process_name = exe_base_name_from_pid(pid).unwrap_or_else(|_| "unknown".to_string());

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn exe_base_name_from_pid(pid: u32) -> Result<String, String> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
        .map_err(|error| format!("OpenProcess 失败：{error}"))?;

    let mut buffer = vec![0u16; MAX_PATH as usize * 8];
    let mut size = buffer.len() as u32;
    let query_result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    let _ = unsafe { CloseHandle(handle) };

    query_result.map_err(|error| format!("QueryFullProcessImageNameW 失败：{error}"))?;

    let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
    let file_name = Path::new(&full_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "无法解析前台进程文件名。".to_string())?;

    Ok(file_name.to_string())
}

fn get_now_playing_powershell() -> Result<MediaInfo, String> {
    let script = r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType=WindowsRuntime]
$async = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]::RequestAsync()
$mgr = $async.GetAwaiter().GetResult()
$session = $mgr.GetCurrentSession()
if (-not $session) { exit 0 }
$info = $session.TryGetMediaPropertiesAsync().GetAwaiter().GetResult()
if (-not $info) { exit 0 }
@{
  title = $info.Title
  artist = $info.Artist
  album = $info.AlbumTitle
  sourceAppId = $session.SourceAppUserModelId
} | ConvertTo-Json -Compress
"#;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|error| format!("启动 PowerShell 失败：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("读取媒体信息失败：{}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(MediaInfo::default());
    }

    let value = serde_json::from_str::<Value>(trimmed)
        .map_err(|error| format!("解析媒体信息失败：{error}"))?;

    Ok(MediaInfo {
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        artist: value
            .get("artist")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        album: value
            .get("album")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        source_app_id: value
            .get("sourceAppId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    })
}

#[cfg(target_os = "windows")]
fn get_now_playing_native() -> Result<MediaInfo, String> {
    const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x80010106u32 as i32);

    unsafe {
        let result = CoInitializeEx(None, COINIT_MULTITHREADED);
        if result.is_err() && result != RPC_E_CHANGED_MODE {
            return Err(format!("初始化 WinRT 失败：{result:?}"));
        }
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|error| format!("请求媒体会话管理器失败：{error}"))?
        .get()
        .map_err(|error| format!("获取媒体会话管理器失败：{error}"))?;

    let session = manager
        .GetCurrentSession()
        .map_err(|error| format!("读取当前媒体会话失败：{error}"))?;

    let source_app_id = session
        .SourceAppUserModelId()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();

    let properties = session
        .TryGetMediaPropertiesAsync()
        .map_err(|error| format!("请求媒体属性失败：{error}"))?
        .get()
        .map_err(|error| format!("读取媒体属性失败：{error}"))?;

    let title = properties
        .Title()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let artist = properties
        .Artist()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let album = properties
        .AlbumTitle()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();

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

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let native_error = match get_now_playing_native() {
        Ok(info) if !info.is_empty() => return Ok(info),
        Ok(_) => None,
        Err(error) => Some(error),
    };

    match get_now_playing_powershell() {
        Ok(info) => Ok(info),
        Err(error) => {
            if let Some(native_error) = native_error {
                Err(format!("原生 WinRT：{native_error}；PowerShell：{error}"))
            } else {
                Err(error)
            }
        }
    }
}

pub fn run_self_test() -> PlatformSelfTestResult {
    let foreground = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            true,
            "前台应用采集正常",
            format!("当前前台应用：{}", snapshot.process_name),
            Vec::new(),
        ),
        Err(error) => make_probe(false, "前台应用采集失败", error, Vec::new()),
    };

    let window_title = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            !snapshot.process_title.trim().is_empty(),
            if snapshot.process_title.trim().is_empty() {
                "窗口标题为空"
            } else {
                "窗口标题采集正常"
            },
            if snapshot.process_title.trim().is_empty() {
                "当前前台窗口没有可用标题。".into()
            } else {
                snapshot.process_title
            },
            Vec::new(),
        ),
        Err(error) => make_probe(false, "窗口标题采集失败", error, Vec::new()),
    };

    let media = match get_now_playing() {
        Ok(info) if !info.is_empty() => make_probe(
            true,
            "媒体采集正常",
            info.summary(),
            Vec::new(),
        ),
        Ok(_) => make_probe(
            true,
            "当前没有播放中的媒体",
            "系统当前没有可用的正在播放信息。".to_string(),
            Vec::new(),
        ),
        Err(error) => make_probe(false, "媒体采集失败", error, Vec::new()),
    };

    build_self_test_result(foreground, window_title, media)
}
