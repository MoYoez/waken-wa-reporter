use std::path::Path;

use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

#[cfg(target_os = "windows")]
use windows::{
    core::{HRESULT, PWSTR},
    Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    Win32::{
        Foundation::{CloseHandle, MAX_PATH},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
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

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err("读取前台窗口失败：GetForegroundWindow 返回空句柄。".into());
    }

    let process_title = if include_process_title {
        let title_len = unsafe { GetWindowTextLengthW(hwnd) };
        if title_len <= 0 {
            String::new()
        } else {
            let mut buffer = vec![0u16; title_len as usize + 1];
            let written = unsafe { GetWindowTextW(hwnd, &mut buffer) };
            String::from_utf16_lossy(&buffer[..written as usize])
        }
    } else {
        String::new()
    };

    let process_name = if include_process_name {
        let mut pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if pid == 0 {
            return Err("读取前台窗口失败：未能解析前台进程 ID。".into());
        }
        exe_base_name_from_pid(pid).unwrap_or_else(|_| "unknown".to_string())
    } else {
        String::new()
    };

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

#[cfg(target_os = "windows")]
struct ComInitGuard {
    should_uninitialize: bool,
}

#[cfg(target_os = "windows")]
impl Drop for ComInitGuard {
    fn drop(&mut self) {
        if self.should_uninitialize {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn init_com_for_media() -> Result<ComInitGuard, String> {
    const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x80010106u32 as i32);

    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

    if result.is_ok() {
        return Ok(ComInitGuard {
            should_uninitialize: true,
        });
    }

    if result == RPC_E_CHANGED_MODE {
        return Ok(ComInitGuard {
            should_uninitialize: false,
        });
    }

    Err(format!("初始化 WinRT 失败：{result:?}"))
}

#[cfg(target_os = "windows")]
fn get_now_playing_native() -> Result<MediaInfo, String> {
    let _com_guard = init_com_for_media()?;

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
    get_now_playing_native()
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
        Ok(info) if !info.is_empty() => {
            make_probe(true, "媒体采集正常", info.summary(), Vec::new())
        }
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

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("当前平台不支持辅助功能权限申请。".into())
}
