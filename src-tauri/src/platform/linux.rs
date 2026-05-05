#[path = "linux/command.rs"]
mod command;
#[path = "linux/media.rs"]
mod media;
#[path = "linux/self_test.rs"]
mod self_test;
#[path = "linux/wayland.rs"]
mod wayland;
#[path = "linux/x11.rs"]
mod x11;

use crate::models::PlatformSelfTestResult;
use crate::platform::{ForegroundSnapshot, MediaInfo};

use super::{build_self_test_result, localized_text, make_probe, ProbeTextSpec};
use command::has_env;
use media::get_now_playing as read_now_playing;
use wayland::{get_foreground_snapshot_wayland, get_foreground_snapshot_wayland_for_reporting};
use x11::{get_foreground_snapshot_x11, get_foreground_snapshot_x11_for_reporting};

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland() {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11() {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11().or_else(|x11_error| {
        get_foreground_snapshot_wayland().map_err(|wayland_error| {
            format!("读取 Linux 前台窗口失败。X11：{x11_error}；Wayland：{wayland_error}")
        })
    })
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland_for_reporting(
            include_process_name,
            include_process_title,
        ) {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11_for_reporting(
                include_process_name,
                include_process_title,
            ) {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11_for_reporting(include_process_name, include_process_title).or_else(
        |x11_error| {
            get_foreground_snapshot_wayland_for_reporting(
                include_process_name,
                include_process_title,
            )
            .map_err(|wayland_error| {
                format!("读取 Linux 前台窗口失败。X11：{x11_error}；Wayland：{wayland_error}")
            })
        },
    )
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
    Err("当前平台不支持辅助功能权限申请。".into())
}
