use serde_json::json;

use crate::models::{PlatformProbeResult, PlatformSelfTestResult};
use crate::platform::MediaInfo;

use super::{
    accessibility_permission_granted, build_self_test_result, get_foreground_snapshot,
    get_foreground_snapshot_for_reporting, get_now_playing, localized_text, make_probe,
    ProbeTextSpec,
};

pub(super) fn run_self_test() -> PlatformSelfTestResult {
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
            localized_text(
                "platformSelfTest.detail.foregroundReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "foreground"),
        ),
    };

    let window_title = match get_foreground_snapshot_for_reporting(false, true) {
        Ok(snapshot) => make_window_title_probe(&snapshot.process_title),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "窗口标题采集失败",
            ),
            localized_text(
                "platformSelfTest.detail.windowTitleReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "window"),
        ),
    };

    let media = match get_now_playing() {
        Ok(info) if !info.is_empty() => make_media_probe(&info),
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
                "如果你正在测试媒体采集，请先播放一段音乐后再运行自检。",
            )],
        ),
        Err(error) => make_probe(
            false,
            localized_text("platformSelfTest.summary.mediaFailed", None, "媒体采集失败"),
            localized_text(
                "platformSelfTest.detail.mediaReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "media"),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

fn make_window_title_probe(process_title: &str) -> PlatformProbeResult {
    make_probe(
        !process_title.trim().is_empty(),
        if process_title.trim().is_empty() {
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
        if process_title.trim().is_empty() {
            if accessibility_permission_granted() {
                localized_text(
                    "platformSelfTest.detail.windowTitleEmpty",
                    None,
                    "当前前台窗口没有可用标题。",
                )
            } else {
                localized_text(
                    "platformSelfTest.detail.windowTitleEmptyPermissionMissing",
                    None,
                    "当前前台窗口没有可用标题，且尚未授予辅助功能权限。",
                )
            }
        } else {
            localized_text(
                "platformSelfTest.detail.windowTitleCurrent",
                Some(json!({ "processTitle": process_title })),
                process_title.to_string(),
            )
        },
        macos_guidance("", "window"),
    )
}

fn make_media_probe(info: &MediaInfo) -> PlatformProbeResult {
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

fn macos_guidance(error: &str, probe: &str) -> Vec<ProbeTextSpec> {
    let lower = error.to_lowercase();
    let mut guidance = Vec::new();

    if probe == "foreground" {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosForegroundBridge",
            None,
            "当前版本的 macOS 前台应用采集只走原生桥接，不再使用 osascript。",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosCheckSystemVersion",
            None,
            "如果仍然失败，请检查系统版本或是否存在窗口列表访问限制。",
        ));
    }

    if probe == "window" {
        if accessibility_permission_granted() {
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionGrantedButNoTitle",
                None,
                "已检测到“辅助功能”权限；如果仍然没有标题，通常是当前应用本身未暴露稳定标题。",
            ));
        } else {
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionRequired",
                None,
                "macOS 窗口标题采集依赖“辅助功能”授权。",
            ));
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionSettings",
                None,
                "可以在设置页点“申请辅助功能权限”，或前往“系统设置 -> 隐私与安全性 -> 辅助功能”手动开启。",
            ));
        }
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosWindowTitleUnstable",
            None,
            "部分应用即使已授权，也可能不会返回稳定的窗口标题。",
        ));
    }

    if probe == "media" || lower.contains("nowplaying-cli") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosInstallNowPlayingCli",
            None,
            "请先安装 nowplaying-cli：`brew install nowplaying-cli`。",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosMediaEmpty",
            None,
            "如果当前没有正在播放的媒体，客户端现在会直接返回空结果，不再记为失败。",
        ));
    }

    if guidance.is_empty() {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosCheckPermissions",
            None,
            "如果这是权限问题，请先检查 macOS 的“辅助功能”和“自动化”授权。",
        ));
    }

    guidance
}
