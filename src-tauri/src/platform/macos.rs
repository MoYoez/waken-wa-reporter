#[path = "macos/bridge.rs"]
mod bridge;
#[path = "macos/command.rs"]
mod command;
#[path = "macos/media.rs"]
mod media;
#[path = "macos/self_test.rs"]
mod self_test;

use super::{
    build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo,
    ProbeTextSpec,
};
use crate::models::PlatformSelfTestResult;
use bridge::{
    accessibility_permission_granted, read_frontmost_app_name, read_frontmost_window_title,
};
use media::get_now_playing as read_now_playing;

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let process_name = read_frontmost_app_name()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "读取 macOS 前台应用失败。".to_string())?;
    let process_title = read_frontmost_window_title().unwrap_or_default();

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
        read_frontmost_window_title().unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name: String::new(),
        process_title,
    })
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    read_now_playing()
}

pub fn get_now_playing_for_reporting(
    include_media: bool,
    include_play_source: bool,
) -> Result<MediaInfo, String> {
    if !include_media && !include_play_source {
        return Ok(MediaInfo::default());
    }

    read_now_playing().map(|media| media.into_reporting_subset(include_media, include_play_source))
}

pub fn get_now_playing_artwork_for_reporting(
    include_play_source: bool,
) -> Result<MediaInfo, String> {
    read_now_playing().map(|media| media.into_reporting_subset(true, include_play_source))
}

pub fn run_self_test() -> PlatformSelfTestResult {
    self_test::run_self_test()
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Ok(bridge::request_accessibility_permission())
}
