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
use crate::models::{AndroidPermissionStatus, PlatformSelfTestResult};

const COLLECTOR_CLASS: &str = "com/waken_wa_reporter_rustc/app/AndroidActivityCollector";
const COLLECTOR_BINARY_NAME: &str = "com.waken_wa_reporter_rustc.app.AndroidActivityCollector";
const ANDROID_INTENT_FLAG_ACTIVITY_NEW_TASK: i32 = 0x10000000;
const ANDROID_SETTINGS_USAGE_ACCESS: &str = "android.settings.USAGE_ACCESS_SETTINGS";
const ANDROID_SETTINGS_NOTIFICATION_LISTENER: &str =
    "android.settings.ACTION_NOTIFICATION_LISTENER_SETTINGS";
const ANDROID_SETTINGS_ACCESSIBILITY: &str = "android.settings.ACCESSIBILITY_SETTINGS";
const ANDROID_SETTINGS_APP_NOTIFICATION: &str = "android.settings.APP_NOTIFICATION_SETTINGS";
const ANDROID_SETTINGS_APPLICATION_DETAILS: &str = "android.settings.APPLICATION_DETAILS_SETTINGS";
const ANDROID_SETTINGS_EXTRA_APP_PACKAGE: &str = "android.provider.extra.APP_PACKAGE";

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
    let _ = request_android_accessibility_permission()?;
    Ok(false)
}

pub fn request_android_usage_access() -> Result<bool, String> {
    let granted = permission_status()?.usage_access_granted;
    open_android_settings(ANDROID_SETTINGS_USAGE_ACCESS)?;
    Ok(granted)
}

pub fn request_android_notification_access() -> Result<bool, String> {
    let granted = permission_status()?.notification_listener_granted;
    open_android_settings(ANDROID_SETTINGS_NOTIFICATION_LISTENER)?;
    Ok(granted)
}

pub fn request_android_accessibility_permission() -> Result<bool, String> {
    open_android_settings(ANDROID_SETTINGS_ACCESSIBILITY)?;
    Ok(false)
}

pub fn open_android_reporter_notification_settings() -> Result<bool, String> {
    match open_android_app_notification_settings() {
        Ok(()) => Ok(false),
        Err(notification_error) => {
            open_android_application_details_settings().map_err(|fallback_error| {
                format!(
                    "打开 Android 通知设置失败：{notification_error}；打开应用设置也失败：{fallback_error}"
                )
            })?;
            Ok(false)
        }
    }
}

pub fn get_permission_status() -> Result<AndroidPermissionStatus, String> {
    permission_status()
}

fn open_android_settings(action: &str) -> Result<(), String> {
    let context = ndk_context::android_context();
    let java_vm = unsafe {
        JavaVM::from_raw(context.vm().cast())
            .map_err(|error| format!("获取 Android JVM 失败：{error}"))?
    };
    let mut env = java_vm
        .attach_current_thread()
        .map_err(|error| format!("附加 Android JVM 线程失败：{error}"))?;
    let action = env
        .new_string(action)
        .map_err(|error| format!("创建 Android 设置 Intent action 失败：{error}"))?;
    let action = JObject::from(action);
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|error| format!("查找 Android Intent 类失败：{error}"))?;
    let intent = env
        .new_object(
            intent_class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(&action)],
        )
        .map_err(|error| format!("创建 Android 设置 Intent 失败：{error}"))?;
    env.call_method(
        &intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(ANDROID_INTENT_FLAG_ACTIVITY_NEW_TASK)],
    )
    .map_err(|error| format!("设置 Android Intent flag 失败：{error}"))?;

    let context = unsafe { JObject::from_raw(context.context().cast()) };
    env.call_method(
        context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )
    .map_err(|error| format!("打开 Android 设置失败：{error}"))?;
    Ok(())
}

fn open_android_app_notification_settings() -> Result<(), String> {
    let context = ndk_context::android_context();
    let java_vm = unsafe {
        JavaVM::from_raw(context.vm().cast())
            .map_err(|error| format!("获取 Android JVM 失败：{error}"))?
    };
    let mut env = java_vm
        .attach_current_thread()
        .map_err(|error| format!("附加 Android JVM 线程失败：{error}"))?;
    let action = env
        .new_string(ANDROID_SETTINGS_APP_NOTIFICATION)
        .map_err(|error| format!("创建 Android 通知设置 Intent action 失败：{error}"))?;
    let action = JObject::from(action);
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|error| format!("查找 Android Intent 类失败：{error}"))?;
    let intent = env
        .new_object(
            intent_class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(&action)],
        )
        .map_err(|error| format!("创建 Android 通知设置 Intent 失败：{error}"))?;
    let package_name = android_package_name(&mut env, context.context().cast())?;
    let package_extra = env
        .new_string(ANDROID_SETTINGS_EXTRA_APP_PACKAGE)
        .map_err(|error| format!("创建 Android 通知设置包名参数失败：{error}"))?;
    let package_extra = JObject::from(package_extra);
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[
            JValue::Object(&package_extra),
            JValue::Object(&package_name),
        ],
    )
    .map_err(|error| format!("设置 Android 通知设置包名失败：{error}"))?;
    start_android_activity(&mut env, context.context().cast(), &intent)
}

fn open_android_application_details_settings() -> Result<(), String> {
    let context = ndk_context::android_context();
    let java_vm = unsafe {
        JavaVM::from_raw(context.vm().cast())
            .map_err(|error| format!("获取 Android JVM 失败：{error}"))?
    };
    let mut env = java_vm
        .attach_current_thread()
        .map_err(|error| format!("附加 Android JVM 线程失败：{error}"))?;
    let action = env
        .new_string(ANDROID_SETTINGS_APPLICATION_DETAILS)
        .map_err(|error| format!("创建 Android 应用设置 Intent action 失败：{error}"))?;
    let action = JObject::from(action);
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|error| format!("查找 Android Intent 类失败：{error}"))?;
    let intent = env
        .new_object(
            intent_class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(&action)],
        )
        .map_err(|error| format!("创建 Android 应用设置 Intent 失败：{error}"))?;
    let package_name = android_package_name(&mut env, context.context().cast())?;
    let scheme = env
        .new_string("package")
        .map_err(|error| format!("创建 Android 应用设置 URI scheme 失败：{error}"))?;
    let scheme = JObject::from(scheme);
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|error| format!("查找 Android Uri 类失败：{error}"))?;
    let uri = env
        .call_static_method(
            uri_class,
            "fromParts",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Landroid/net/Uri;",
            &[
                JValue::Object(&scheme),
                JValue::Object(&package_name),
                JValue::Object(&JObject::null()),
            ],
        )
        .and_then(|value| value.l())
        .map_err(|error| format!("创建 Android 应用设置 URI 失败：{error}"))?;
    env.call_method(
        &intent,
        "setData",
        "(Landroid/net/Uri;)Landroid/content/Intent;",
        &[JValue::Object(&uri)],
    )
    .map_err(|error| format!("设置 Android 应用设置 URI 失败：{error}"))?;
    start_android_activity(&mut env, context.context().cast(), &intent)
}

fn android_package_name<'local>(
    env: &mut jni::JNIEnv<'local>,
    context: jobject,
) -> Result<JObject<'local>, String> {
    let context = unsafe { JObject::from_raw(context) };
    env.call_method(context, "getPackageName", "()Ljava/lang/String;", &[])
        .and_then(|value| value.l())
        .map_err(|error| format!("读取 Android 包名失败：{error}"))
}

fn start_android_activity(
    env: &mut jni::JNIEnv<'_>,
    context: jobject,
    intent: &JObject<'_>,
) -> Result<(), String> {
    env.call_method(
        intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(ANDROID_INTENT_FLAG_ACTIVITY_NEW_TASK)],
    )
    .map_err(|error| format!("设置 Android Intent flag 失败：{error}"))?;

    let context = unsafe { JObject::from_raw(context) };
    match env.call_method(
        context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(intent)],
    ) {
        Ok(_) => Ok(()),
        Err(error) => {
            let _ = clear_pending_exception(env);
            Err(format!("打开 Android 设置失败：{error}"))
        }
    }
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
        .map_err(|error| {
            let _ = clear_pending_exception(&mut env);
            format!("调用 Android 采集器 {method} 失败：{error}")
        })?;
    let object = value.l().map_err(|error| {
        let _ = clear_pending_exception(&mut env);
        format!("读取 Android 采集器 {method} 返回值失败：{error}")
    })?;
    let string = JString::from(object);
    env.get_string(&string)
        .map(|value| value.into())
        .map_err(|error| {
            let _ = clear_pending_exception(&mut env);
            format!("转换 Android 采集器 {method} 返回字符串失败：{error}")
        })
}

fn load_collector_class<'local>(
    env: &mut jni::JNIEnv<'local>,
    context: jobject,
) -> Result<JClass<'local>, String> {
    match env.find_class(COLLECTOR_CLASS) {
        Ok(class) => return Ok(class),
        Err(find_error) => {
            clear_pending_exception(env).map_err(|error| {
                format!("直接查找 Android 采集器失败：{find_error}；清除 JNI 异常失败：{error}")
            })?;
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

fn clear_pending_exception(env: &mut jni::JNIEnv<'_>) -> Result<(), jni::errors::Error> {
    if env.exception_check()? {
        env.exception_clear()?;
    }
    Ok(())
}

fn jni_bool(value: bool) -> jboolean {
    if value {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
