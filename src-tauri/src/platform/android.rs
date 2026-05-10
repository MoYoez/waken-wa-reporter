use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::{jboolean, jobject, JNI_FALSE, JNI_TRUE},
    JavaVM,
};
use serde::Deserialize;
use serde_json::json;

use super::{
    build_self_test_result, localized_text, make_probe, media_timestamps_from_position,
    now_unix_millis_i64, DevicePowerInfo, ForegroundSnapshot, MediaInfo,
};
use crate::models::PlatformSelfTestResult;

const COLLECTOR_CLASS: &str = "com/waken_wa_reporter_rustc/app/AndroidActivityCollector";
const COLLECTOR_BINARY_NAME: &str = "com.waken_wa_reporter_rustc.app.AndroidActivityCollector";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AndroidPermissionStatus {
    usage_access_granted: bool,
    notification_listener_granted: bool,
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AndroidForegroundSnapshot {
    process_name: String,
    process_title: String,
    usage_access_granted: bool,
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AndroidMediaInfo {
    title: String,
    artist: String,
    album: String,
    source_app_id: String,
    source_app_name: String,
    cover_data_url: String,
    app_icon_data_url: String,
    playback_state: String,
    position_ms: Option<i64>,
    duration_ms: Option<i64>,
    reported_at_ms: Option<i64>,
    notification_listener_granted: bool,
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AndroidPowerInfo {
    battery_level: Option<i64>,
    is_charging: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AndroidPermissionRequestResult {
    granted: bool,
    error: Option<String>,
}

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_for_reporting(true, true)
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let snapshot = call_foreground_snapshot(include_process_name, include_process_title)?;
    if let Some(error) = snapshot.error.filter(|value| !value.trim().is_empty()) {
        return Err(error);
    }
    if !snapshot.usage_access_granted {
        return Ok(ForegroundSnapshot::default());
    }

    Ok(ForegroundSnapshot {
        process_name: snapshot.process_name,
        process_title: snapshot.process_title,
    })
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    get_now_playing_for_reporting(true, true)
}

pub fn get_now_playing_for_reporting(
    include_media: bool,
    include_play_source: bool,
) -> Result<MediaInfo, String> {
    read_now_playing(include_media, include_play_source, false, false)
}

pub fn get_now_playing_artwork_for_reporting(
    include_play_source: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, String> {
    read_now_playing(true, include_play_source, true, include_source_icon)
}

pub fn get_device_power_info_for_reporting() -> Result<DevicePowerInfo, String> {
    let raw = call_static_string("getPowerInfo", "()Ljava/lang/String;", &[])?;
    let parsed: AndroidPowerInfo = serde_json::from_str(&raw)
        .map_err(|error| format!("解析 Android 电量信息失败：{error}；原始数据：{raw}"))?;

    Ok(DevicePowerInfo {
        battery_level: parsed.battery_level,
        is_charging: parsed.is_charging,
    })
}

pub fn run_self_test() -> PlatformSelfTestResult {
    let permissions = permission_status().unwrap_or_default();
    let foreground_result = call_foreground_snapshot(true, true);
    let foreground = match foreground_result {
        Ok(snapshot)
            if snapshot.usage_access_granted && !snapshot.process_name.trim().is_empty() =>
        {
            make_probe(
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
            )
        }
        Ok(snapshot) if !snapshot.usage_access_granted || !permissions.usage_access_granted => {
            make_probe(
                false,
                localized_text(
                    "platformSelfTest.summary.foregroundFailed",
                    None,
                    "前台应用采集失败",
                ),
                localized_text(
                    "platformSelfTest.detail.androidUsageAccessRequired",
                    None,
                    "Android 前台应用采集需要开启“使用情况访问权限”。",
                ),
                vec![localized_text(
                    "platformSelfTest.guidance.androidUsageAccessSettings",
                    None,
                    "点“打开权限设置”，为 Waken-Wa 允许使用情况访问。",
                )],
            )
        }
        Ok(_) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.foregroundFailed",
                None,
                "前台应用采集失败",
            ),
            localized_text(
                "platformSelfTest.detail.foregroundReadFailed",
                None,
                "无法读取前台应用。",
            ),
            vec![localized_text(
                "platformSelfTest.guidance.androidOpenRecentApp",
                None,
                "打开一个应用后回到 Waken-Wa 再检测。",
            )],
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

    let window_title = match call_foreground_snapshot(false, true) {
        Ok(snapshot)
            if snapshot.usage_access_granted && !snapshot.process_title.trim().is_empty() =>
        {
            make_probe(
                true,
                localized_text(
                    "platformSelfTest.summary.windowTitleOk",
                    None,
                    "窗口标题采集正常",
                ),
                localized_text(
                    "platformSelfTest.detail.androidWindowTitleFallback",
                    Some(json!({ "processTitle": snapshot.process_title.clone() })),
                    format!("Android 当前以应用名称作为标题：{}", snapshot.process_title),
                ),
                vec![localized_text(
                    "platformSelfTest.guidance.androidWindowTitleFallback",
                    None,
                    "为减少打扰，当前版本不默认启用无障碍读取页面标题。",
                )],
            )
        }
        Ok(snapshot) if !snapshot.usage_access_granted => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "窗口标题采集失败",
            ),
            localized_text(
                "platformSelfTest.detail.androidUsageAccessRequired",
                None,
                "Android 前台应用采集需要开启“使用情况访问权限”。",
            ),
            vec![localized_text(
                "platformSelfTest.guidance.androidUsageAccessSettings",
                None,
                "点“打开权限设置”，为 Waken-Wa 允许使用情况访问。",
            )],
        ),
        Ok(_) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleEmpty",
                None,
                "窗口标题为空",
            ),
            localized_text(
                "platformSelfTest.detail.windowTitleEmpty",
                None,
                "当前前台窗口没有可用标题。",
            ),
            vec![localized_text(
                "platformSelfTest.guidance.androidWindowTitleFallback",
                None,
                "为减少打扰，当前版本不默认启用无障碍读取页面标题。",
            )],
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

    let media_result = call_media_info(true, true, false, false);
    let media = match media_result {
        Ok(info)
            if !info.notification_listener_granted
                || !permissions.notification_listener_granted =>
        {
            make_probe(
                false,
                localized_text("platformSelfTest.summary.mediaFailed", None, "媒体采集失败"),
                localized_text(
                    "platformSelfTest.detail.androidNotificationAccessRequired",
                    None,
                    "Android 媒体采集需要开启“通知使用权”。",
                ),
                vec![localized_text(
                    "platformSelfTest.guidance.androidNotificationAccessSettings",
                    None,
                    "点“打开权限设置”，为 Waken-Wa 允许通知使用权。",
                )],
            )
        }
        Ok(info) => {
            let media = media_info_from_android(info);
            if media.is_empty() {
                make_probe(
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
                        "先播放媒体再重试。",
                    )],
                )
            } else {
                make_probe(
                    true,
                    localized_text("platformSelfTest.summary.mediaOk", None, "媒体采集正常"),
                    localized_text(
                        "platformSelfTest.detail.mediaCurrent",
                        Some(json!({ "mediaSummary": media.summary() })),
                        media.summary(),
                    ),
                    Vec::new(),
                )
            }
        }
        Err(error) => make_probe(
            false,
            localized_text("platformSelfTest.summary.mediaFailed", None, "媒体采集失败"),
            localized_text("platformSelfTest.detail.mediaReadFailed", None, error),
            Vec::new(),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    let raw = call_static_string(
        "openRequiredPermissionSettings",
        "()Ljava/lang/String;",
        &[],
    )?;
    let parsed: AndroidPermissionRequestResult = serde_json::from_str(&raw)
        .map_err(|error| format!("解析 Android 权限跳转结果失败：{error}；原始数据：{raw}"))?;
    if let Some(error) = parsed.error.filter(|value| !value.trim().is_empty()) {
        return Err(error);
    }
    Ok(parsed.granted)
}

fn read_now_playing(
    include_media: bool,
    include_play_source: bool,
    include_artwork: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, String> {
    let info = call_media_info(
        include_media,
        include_play_source,
        include_artwork,
        include_source_icon,
    )?;
    if let Some(error) = info.error.as_ref().filter(|value| !value.trim().is_empty()) {
        return Err(error.clone());
    }
    if !info.notification_listener_granted {
        return Ok(MediaInfo::default());
    }

    Ok(media_info_from_android(info))
}

fn media_info_from_android(info: AndroidMediaInfo) -> MediaInfo {
    let reported_at_ms = info.reported_at_ms.unwrap_or_else(now_unix_millis_i64);
    let playback_state = info.playback_state.trim().to_string();
    let (start_timestamp_ms, end_timestamp_ms) = media_timestamps_from_position(
        &playback_state,
        info.position_ms,
        info.duration_ms,
        reported_at_ms,
    );

    MediaInfo {
        title: info.title,
        artist: info.artist,
        album: info.album,
        source_app_id: info.source_app_id,
        source_app_name: info.source_app_name,
        cover_url: info.cover_data_url,
        source_icon_url: info.app_icon_data_url,
        playback_state,
        position_ms: info.position_ms,
        duration_ms: info.duration_ms,
        start_timestamp_ms,
        end_timestamp_ms,
        reported_at_ms: Some(reported_at_ms),
        genre: String::new(),
    }
}

fn permission_status() -> Result<AndroidPermissionStatus, String> {
    let raw = call_static_string("getPermissionStatus", "()Ljava/lang/String;", &[])?;
    let parsed: AndroidPermissionStatus = serde_json::from_str(&raw)
        .map_err(|error| format!("解析 Android 权限状态失败：{error}；原始数据：{raw}"))?;
    if let Some(error) = parsed
        .error
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        return Err(error.clone());
    }
    Ok(parsed)
}

fn call_foreground_snapshot(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<AndroidForegroundSnapshot, String> {
    let raw = call_static_string(
        "getForegroundSnapshot",
        "(ZZ)Ljava/lang/String;",
        &[
            JValue::Bool(jni_bool(include_process_name)),
            JValue::Bool(jni_bool(include_process_title)),
        ],
    )?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("解析 Android 前台应用信息失败：{error}；原始数据：{raw}"))
}

fn call_media_info(
    include_media: bool,
    include_play_source: bool,
    include_artwork: bool,
    include_source_icon: bool,
) -> Result<AndroidMediaInfo, String> {
    let raw = call_static_string(
        "getNowPlaying",
        "(ZZZZ)Ljava/lang/String;",
        &[
            JValue::Bool(jni_bool(include_media)),
            JValue::Bool(jni_bool(include_play_source)),
            JValue::Bool(jni_bool(include_artwork)),
            JValue::Bool(jni_bool(include_source_icon)),
        ],
    )?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("解析 Android 媒体信息失败：{error}；原始数据：{raw}"))
}

fn call_static_string(
    method: &str,
    signature: &str,
    args: &[JValue<'_, '_>],
) -> Result<String, String> {
    let context = ndk_context::android_context();
    let java_vm = unsafe {
        JavaVM::from_raw(context.vm().cast())
            .map_err(|error| format!("获取 Android JVM 失败：{error}"))?
    };
    let mut env = java_vm
        .attach_current_thread()
        .map_err(|error| format!("附加 Android JVM 线程失败：{error}"))?;
    let class = load_collector_class(&mut env, context.context().cast())?;
    let value = env
        .call_static_method(class, method, signature, args)
        .map_err(|error| format!("调用 Android 采集器 {method} 失败：{error}"))?;
    let object = value
        .l()
        .map_err(|error| format!("读取 Android 采集器 {method} 返回值失败：{error}"))?;
    let string = JString::from(object);
    env.get_string(&string)
        .map(|value| value.into())
        .map_err(|error| format!("转换 Android 采集器 {method} 返回字符串失败：{error}"))
}

fn load_collector_class<'local>(
    env: &mut jni::JNIEnv<'local>,
    context: jobject,
) -> Result<JClass<'local>, String> {
    match env.find_class(COLLECTOR_CLASS) {
        Ok(class) => return Ok(class),
        Err(find_error) => {
            let context = unsafe { JObject::from_raw(context) };
            let class_loader = env
                .call_method(context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
                .and_then(|value| value.l())
                .map_err(|error| {
                    format!("查找 Android 采集器失败：{find_error}；读取 ClassLoader 失败：{error}")
                })?;
            let class_name = env
                .new_string(COLLECTOR_BINARY_NAME)
                .map_err(|error| format!("创建 Android 采集器类名失败：{error}"))?;
            let class_name = JObject::from(class_name);
            let class = env
                .call_method(
                    class_loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[JValue::Object(&class_name)],
                )
                .and_then(|value| value.l())
                .map_err(|error| format!("通过 ClassLoader 查找 Android 采集器失败：{error}"))?;
            Ok(JClass::from(class))
        }
    }
}

fn jni_bool(value: bool) -> jboolean {
    if value {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
