use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    mem::size_of,
    path::Path,
    sync::{Mutex, OnceLock},
};

use serde_json::json;

use super::{
    build_self_test_result, localized_text, make_probe, media_timestamps_from_position,
    now_unix_millis_i64, ForegroundSnapshot, MediaInfo,
};
use crate::models::PlatformSelfTestResult;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
#[cfg(target_os = "windows")]
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};

#[cfg(target_os = "windows")]
use windows::{
    core::{HRESULT, HSTRING, PCWSTR, PWSTR},
    ApplicationModel::AppInfo,
    Foundation::Size,
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::{DataReader, RandomAccessStreamReference},
    Win32::{
        Foundation::{CloseHandle, MAX_PATH},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
        },
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{
                DestroyIcon, DrawIconEx, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
                GetWindowThreadProcessId, PrivateExtractIconsW, DI_NORMAL, HICON,
            },
        },
    },
};
#[cfg(target_os = "windows")]
std::thread_local! {
    static MEDIA_SESSION_MANAGER: RefCell<Option<GlobalSystemMediaTransportControlsSessionManager>> =
        const { RefCell::new(None) };
}

const APP_ICON_SIZE: i32 = 256;

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
fn get_cached_media_session_manager(
) -> Result<GlobalSystemMediaTransportControlsSessionManager, String> {
    MEDIA_SESSION_MANAGER.with(|cache| {
        if let Some(manager) = cache.borrow().as_ref().cloned() {
            return Ok(manager);
        }

        let request = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .map_err(|error| format!("请求媒体会话管理器失败：{error}"))?;
        let manager = request
            .join()
            .map_err(|error| format!("获取媒体会话管理器失败：{error}"))?;

        *cache.borrow_mut() = Some(manager.clone());
        Ok(manager)
    })
}

#[cfg(target_os = "windows")]
fn clear_cached_media_session_manager() {
    MEDIA_SESSION_MANAGER.with(|cache| {
        cache.borrow_mut().take();
    });
}

#[cfg(target_os = "windows")]
fn get_now_playing_native(
    include_media: bool,
    include_play_source: bool,
    include_artwork: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, String> {
    let _com_guard = init_com_for_media()?;

    let manager = get_cached_media_session_manager()?;
    let session = match manager.GetCurrentSession() {
        Ok(session) => session,
        Err(_) => {
            clear_cached_media_session_manager();
            get_cached_media_session_manager()?
                .GetCurrentSession()
                .map_err(|error| format!("读取当前媒体会话失败：{error}"))?
        }
    };

    let source_app_id = if include_play_source {
        session
            .SourceAppUserModelId()
            .ok()
            .map(|value| value.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let playback_state = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(playback_status_to_string)
        .unwrap_or_default();
    let (position_ms, duration_ms) = session
        .GetTimelineProperties()
        .ok()
        .map(|timeline| {
            let position_ms = timeline.Position().ok().and_then(timespan_to_ms);
            let start_ms = timeline
                .StartTime()
                .ok()
                .and_then(timespan_to_ms)
                .unwrap_or(0);
            let end_ms = timeline
                .EndTime()
                .ok()
                .and_then(timespan_to_ms)
                .unwrap_or(0);
            let duration_ms = if end_ms > start_ms {
                Some(end_ms - start_ms)
            } else {
                None
            };
            (position_ms, duration_ms)
        })
        .unwrap_or((None, None));
    let reported_at_ms = now_unix_millis_i64();
    let (start_timestamp_ms, end_timestamp_ms) =
        media_timestamps_from_position(&playback_state, position_ms, duration_ms, reported_at_ms);

    let (title, artist, album, cover_url, genre) = if include_media {
        let request = session
            .TryGetMediaPropertiesAsync()
            .map_err(|error| format!("请求媒体属性失败：{error}"))?;
        let properties = request
            .join()
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
        let cover_url = if include_artwork {
            read_media_artwork(&properties).unwrap_or_default()
        } else {
            String::new()
        };
        let genre = read_media_genre(&properties).unwrap_or_default();

        (title, artist, album, cover_url, genre)
    } else {
        (String::new(), String::new(), String::new(), String::new(), String::new())
    };
    let source_icon_url = if include_play_source && include_source_icon {
        read_source_app_icon_data_url(&source_app_id).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        cover_url,
        source_icon_url,
        playback_state,
        position_ms,
        duration_ms,
        start_timestamp_ms,
        end_timestamp_ms,
        reported_at_ms: Some(reported_at_ms),
        genre,
    }
    .into_reporting_subset(include_media, include_play_source))
}

fn playback_status_to_string(
    status: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
) -> String {
    match status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "playing".into(),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "paused".into(),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => "stopped".into(),
        _ => String::new(),
    }
}

fn timespan_to_ms(value: windows::Foundation::TimeSpan) -> Option<i64> {
    let ticks = value.Duration;
    if ticks < 0 {
        return None;
    }
    Some(ticks / 10_000)
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let media = get_now_playing_native(true, true, false, false)?;
    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

pub fn get_now_playing_for_reporting(
    include_media: bool,
    include_play_source: bool,
) -> Result<MediaInfo, String> {
    if !include_media && !include_play_source {
        return Ok(MediaInfo::default());
    }

    get_now_playing_native(include_media, include_play_source, false, false)
}

pub fn get_now_playing_artwork_for_reporting(
    include_play_source: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, String> {
    get_now_playing_native(true, include_play_source, true, include_source_icon)
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
            localized_text("platformSelfTest.detail.foregroundReadFailed", None, error),
            Vec::new(),
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
            localized_text("platformSelfTest.detail.windowTitleReadFailed", None, error),
            Vec::new(),
        ),
    };

    let media = match get_now_playing() {
        Ok(info) if !info.is_empty() => make_probe(
            true,
            localized_text("platformSelfTest.summary.mediaOk", None, "媒体采集正常"),
            localized_text(
                "platformSelfTest.detail.mediaCurrent",
                Some(json!({ "mediaSummary": info.summary() })),
                info.summary(),
            ),
            Vec::new(),
        ),
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
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text("platformSelfTest.summary.mediaFailed", None, "媒体采集失败"),
            localized_text("platformSelfTest.detail.mediaReadFailed", None, error),
            Vec::new(),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

fn read_media_genre(
    properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<String, String> {
    let genres = properties
        .Genres()
        .map_err(|e| format!("请求媒体流派失败：{e}"))?;
    let size = genres.Size().map_err(|e| format!("获取流派数量失败：{e}"))?;
    if size == 0 {
        return Ok(String::new());
    }
    let mut parts = Vec::with_capacity(size as usize);
    for i in 0..size {
        let v = genres.GetAt(i).map_err(|e| format!("获取流派失败：{e}"))?;
        parts.push(v.to_string());
    }
    Ok(parts.join(", "))
}

fn read_media_artwork(
    properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<String, String> {
    let thumbnail = properties
        .Thumbnail()
        .map_err(|e| format!("请求媒体缩略图失败：{e}"))?;
    let request = thumbnail
        .OpenReadAsync()
        .map_err(|e| format!("请求媒体缩略图流失败：{e}"))?;
    let stream = request
        .join()
        .map_err(|e| format!("读取媒体缩略图流失败：{e}"))?;

    let size = stream
        .Size()
        .map_err(|e| format!("读取媒体缩略图大小失败：{e}"))? as u32;
    if size == 0 {
        return Ok(String::new());
    }

    let input_stream = stream
        .GetInputStreamAt(0)
        .map_err(|e| format!("获取输入流失败：{e}"))?;
    let reader =
        DataReader::CreateDataReader(&input_stream).map_err(|e| format!("创建读取器失败：{e}"))?;
    let request = reader
        .LoadAsync(size)
        .map_err(|e| format!("请求缩略图缓冲区失败：{e}"))?;
    request
        .join()
        .map_err(|e| format!("读取缩略图缓冲区失败：{e}"))?;

    let mut bytes = vec![0u8; size as usize];
    reader
        .ReadBytes(&mut bytes)
        .map_err(|e| format!("读取缩略图字节失败：{e}"))?;

    if bytes.is_empty() {
        return Ok(String::new());
    }

    let content_type = stream
        .ContentType()
        .ok()
        .map(|v| v.to_string())
        .filter(|v| v.starts_with("image/"))
        .unwrap_or_else(|| "image/jpeg".to_string());

    let encoded = BASE64_STANDARD.encode(&bytes);
    Ok(format!("data:{content_type};base64,{encoded}"))
}

fn source_icon_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn read_source_app_icon_data_url(source_app_id: &str) -> Option<String> {
    let cache_key = source_app_id.trim();
    if cache_key.is_empty() {
        return None;
    }

    if let Some(cached) = source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(cache_key)
        .cloned()
    {
        return (!cached.is_empty()).then_some(cached);
    }

    let icon = read_packaged_app_icon_data_url(cache_key)
        .or_else(|| read_process_app_icon_data_url(cache_key).ok().flatten())
        .unwrap_or_default();

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    (!icon.is_empty()).then_some(icon)
}

fn read_packaged_app_icon_data_url(source_app_id: &str) -> Option<String> {
    let app_info = AppInfo::GetFromAppUserModelId(&HSTRING::from(source_app_id)).ok()?;
    let display_info = app_info.DisplayInfo().ok()?;
    let logo_stream = display_info
        .GetLogo(Size {
            Width: APP_ICON_SIZE as f32,
            Height: APP_ICON_SIZE as f32,
        })
        .ok()?;

    read_stream_reference_data_url(&logo_stream).ok().flatten()
}

fn read_process_app_icon_data_url(source_app_id: &str) -> Result<Option<String>, String> {
    let executable_path = match resolve_process_image_path_from_source_app_id(source_app_id) {
        Some(path) => path,
        None => return Ok(None),
    };
    let bytes = render_executable_icon_png(&executable_path, APP_ICON_SIZE)?;
    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(bytes)
    )))
}

fn read_stream_reference_data_url(
    reference: &RandomAccessStreamReference,
) -> Result<Option<String>, String> {
    let request = reference
        .OpenReadAsync()
        .map_err(|error| format!("请求应用图标流失败：{error}"))?;
    let stream = request
        .join()
        .map_err(|error| format!("读取应用图标流失败：{error}"))?;

    let size = stream
        .Size()
        .map_err(|error| format!("读取应用图标大小失败：{error}"))? as u32;
    if size == 0 {
        return Ok(None);
    }

    let input_stream = stream
        .GetInputStreamAt(0)
        .map_err(|error| format!("获取应用图标输入流失败：{error}"))?;
    let reader = DataReader::CreateDataReader(&input_stream)
        .map_err(|error| format!("创建应用图标读取器失败：{error}"))?;
    let request = reader
        .LoadAsync(size)
        .map_err(|error| format!("请求应用图标缓冲区失败：{error}"))?;
    request
        .join()
        .map_err(|error| format!("加载应用图标缓冲区失败：{error}"))?;

    let mut bytes = vec![0u8; size as usize];
    reader
        .ReadBytes(&mut bytes)
        .map_err(|error| format!("读取应用图标字节失败：{error}"))?;

    if bytes.is_empty() {
        return Ok(None);
    }

    let content_type = stream
        .ContentType()
        .ok()
        .map(|value| value.to_string())
        .filter(|value| value.starts_with("image/"))
        .unwrap_or_else(|| "image/png".to_string());

    Ok(Some(format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    )))
}

fn process_image_path_from_pid(pid: u32) -> Result<String, String> {
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

    Ok(String::from_utf16_lossy(&buffer[..size as usize]))
}

fn resolve_process_image_path_from_source_app_id(source_app_id: &str) -> Option<String> {
    let trimmed = source_app_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    if (trimmed.contains('\\') || trimmed.contains('/')) && Path::new(trimmed).exists() {
        return Some(trimmed.to_string());
    }

    let candidates = process_name_candidates(trimmed);
    if candidates.is_empty() {
        return None;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut path = None;
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        loop {
            let executable_name = utf16z_to_string(&entry.szExeFile);
            if matches_process_candidate(&executable_name, &candidates) {
                if let Ok(executable_path) = process_image_path_from_pid(entry.th32ProcessID) {
                    path = Some(executable_path);
                    break;
                }
            }

            if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                break;
            }
        }
    }

    let _ = unsafe { CloseHandle(snapshot) };
    path
}

fn process_name_candidates(source_app_id: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut push_candidate = |value: &str| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return;
        }
        if !candidates
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            candidates.push(trimmed.to_string());
        }
    };

    let tail = source_app_id
        .trim()
        .rsplit(['\\', '/', '!'])
        .next()
        .unwrap_or(source_app_id.trim());
    let tail = tail.split('_').next().unwrap_or(tail);
    push_candidate(tail);

    let dotted_tail = tail.rsplit('.').next().unwrap_or(tail);
    push_candidate(dotted_tail);

    if !tail.to_ascii_lowercase().ends_with(".exe") {
        push_candidate(&format!("{tail}.exe"));
    }
    if !dotted_tail.to_ascii_lowercase().ends_with(".exe") {
        push_candidate(&format!("{dotted_tail}.exe"));
    }

    candidates
}

fn matches_process_candidate(executable_name: &str, candidates: &[String]) -> bool {
    candidates
        .iter()
        .any(|candidate| executable_name.eq_ignore_ascii_case(candidate))
}

fn utf16z_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

fn render_executable_icon_png(executable_path: &str, target_size: i32) -> Result<Vec<u8>, String> {
    if let Some(hicon) = extract_executable_icon(executable_path, target_size) {
        let render_result = render_hicon_png(hicon, target_size);
        let _ = unsafe { DestroyIcon(hicon) };
        return render_result;
    }

    let wide_path = encode_wide(executable_path);
    let mut file_info = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            Default::default(),
            Some(&mut file_info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if result == 0 || file_info.hIcon.is_invalid() {
        return Err("读取应用图标句柄失败。".to_string());
    }

    let hicon = file_info.hIcon;
    let render_result = render_hicon_png(hicon, target_size);
    let _ = unsafe { DestroyIcon(hicon) };
    render_result
}

fn extract_executable_icon(executable_path: &str, target_size: i32) -> Option<HICON> {
    let wide_path = encode_wide(executable_path);
    if wide_path.len() > MAX_PATH as usize {
        return None;
    }

    let mut fixed_path = [0u16; MAX_PATH as usize];
    let path_len = wide_path.len().min(fixed_path.len());
    fixed_path[..path_len].copy_from_slice(&wide_path[..path_len]);

    let mut icons = [HICON::default(); 1];
    let extracted = unsafe {
        PrivateExtractIconsW(
            &fixed_path,
            0,
            target_size,
            target_size,
            Some(&mut icons),
            None,
            0,
        )
    };
    (extracted > 0 && !icons[0].is_invalid()).then_some(icons[0])
}

fn render_hicon_png(hicon: HICON, target_size: i32) -> Result<Vec<u8>, String> {
    let screen_dc = unsafe { GetDC(None) };
    if screen_dc.is_invalid() {
        return Err("创建屏幕绘图上下文失败。".to_string());
    }

    let memory_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    if memory_dc.is_invalid() {
        let _ = unsafe { ReleaseDC(None, screen_dc) };
        return Err("创建内存绘图上下文失败。".to_string());
    }

    let mut bitmap_info = BITMAPINFO::default();
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = target_size;
    bitmap_info.bmiHeader.biHeight = -target_size;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB.0;

    let mut bits_ptr = std::ptr::null_mut::<c_void>();
    let bitmap = unsafe {
        CreateDIBSection(
            Some(screen_dc),
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        )
    }
    .map_err(|error| format!("创建应用图标位图失败：{error}"))?;

    let old_object = unsafe { SelectObject(memory_dc, HGDIOBJ(bitmap.0)) };
    let draw_result = unsafe {
        DrawIconEx(
            memory_dc,
            0,
            0,
            hicon,
            target_size,
            target_size,
            0,
            None,
            DI_NORMAL,
        )
    };

    let _ = unsafe { SelectObject(memory_dc, old_object) };
    let _ = unsafe { DeleteDC(memory_dc) };
    let _ = unsafe { ReleaseDC(None, screen_dc) };

    draw_result.map_err(|error| format!("绘制应用图标失败：{error}"))?;
    if bits_ptr.is_null() {
        let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        return Err("应用图标位图缓冲区为空。".to_string());
    }

    let pixel_len = (target_size as usize)
        .saturating_mul(target_size as usize)
        .saturating_mul(4);
    let raw_bgra = unsafe { std::slice::from_raw_parts(bits_ptr as *const u8, pixel_len) };
    let mut rgba = raw_bgra.to_vec();
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };

    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(
            &rgba,
            target_size as u32,
            target_size as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(|error| format!("编码应用图标 PNG 失败：{error}"))?;

    Ok(png)
}

fn encode_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("当前平台不支持辅助功能权限申请。".into())
}
