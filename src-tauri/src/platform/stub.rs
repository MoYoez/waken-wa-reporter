use super::{build_self_test_result, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    Err("当前平台暂不支持实时采集。".into())
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    Ok(MediaInfo::default())
}

pub fn run_self_test() -> PlatformSelfTestResult {
    build_self_test_result(
        make_probe(
            false,
            "前台应用采集不支持",
            "当前平台暂不支持实时采集。",
            Vec::new(),
        ),
        make_probe(
            false,
            "窗口标题采集不支持",
            "当前平台暂不支持实时采集。",
            Vec::new(),
        ),
        make_probe(
            false,
            "媒体采集不支持",
            "当前平台暂不支持实时采集。",
            Vec::new(),
        ),
    )
}
