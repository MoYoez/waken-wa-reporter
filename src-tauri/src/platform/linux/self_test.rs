use serde_json::json;

use crate::models::PlatformSelfTestResult;
use crate::platform::{ForegroundSnapshot, MediaInfo};

use super::{
    build_self_test_result, get_foreground_snapshot, get_now_playing, localized_text, make_probe,
    ProbeTextSpec,
};

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

        return localized_text("platformSelfTest.detail.linuxMediaUnavailable", None, error);
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

pub(super) fn run_self_test() -> PlatformSelfTestResult {
    let foreground = match get_foreground_snapshot() {
        Ok(snapshot) => make_foreground_probe(&snapshot),
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
        Ok(snapshot) => make_window_title_probe(&snapshot),
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

fn make_foreground_probe(snapshot: &ForegroundSnapshot) -> crate::models::PlatformProbeResult {
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

fn make_window_title_probe(snapshot: &ForegroundSnapshot) -> crate::models::PlatformProbeResult {
    make_probe(
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
                snapshot.process_title.clone(),
            )
        },
        Vec::new(),
    )
}

fn make_media_probe(info: &MediaInfo) -> crate::models::PlatformProbeResult {
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
