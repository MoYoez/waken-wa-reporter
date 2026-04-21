use super::{build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    Err("当前平台暂不支持实时采集。".into())
}

pub fn get_foreground_snapshot_for_reporting(
    _include_process_name: bool,
    _include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    Ok(ForegroundSnapshot::default())
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    Ok(MediaInfo::default())
}

pub fn get_now_playing_for_reporting(
    _include_media: bool,
    _include_play_source: bool,
) -> Result<MediaInfo, String> {
    Ok(MediaInfo::default())
}

pub fn run_self_test() -> PlatformSelfTestResult {
    build_self_test_result(
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.foregroundFailed",
                None,
                "前台应用采集不支持",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "当前平台暂不支持实时采集。",
            ),
            Vec::new(),
        ),
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "窗口标题采集不支持",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "当前平台暂不支持实时采集。",
            ),
            Vec::new(),
        ),
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.mediaFailed",
                None,
                "媒体采集不支持",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "当前平台暂不支持实时采集。",
            ),
            Vec::new(),
        ),
    )
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("当前平台不支持辅助功能权限申请。".into())
}
