#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

use serde_json::{Map, Value};

use crate::models::{PlatformProbeResult, PlatformSelfTestResult};

#[derive(Clone, Debug)]
pub struct ForegroundSnapshot {
    pub process_name: String,
    pub process_title: String,
}

#[derive(Clone, Debug, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source_app_id: String,
}

impl MediaInfo {
    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty() && self.artist.trim().is_empty() && self.album.trim().is_empty()
    }

    pub fn signature(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        format!(
            "{}\u{1e}{}\u{1e}{}\u{1e}{}",
            self.title.trim(),
            self.artist.trim(),
            self.album.trim(),
            self.source_app_id.trim()
        )
    }

    pub fn as_metadata_map(&self) -> Option<Map<String, Value>> {
        if self.is_empty() {
            return None;
        }

        let mut map = Map::new();
        if !self.title.trim().is_empty() {
            map.insert("title".into(), Value::String(self.title.trim().to_string()));
        }
        if !self.artist.trim().is_empty() {
            map.insert("artist".into(), Value::String(self.artist.trim().to_string()));
            map.insert("singer".into(), Value::String(self.artist.trim().to_string()));
        }
        if !self.album.trim().is_empty() {
            map.insert("album".into(), Value::String(self.album.trim().to_string()));
        }
        Some(map)
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.title.trim().is_empty() {
            parts.push(self.title.trim().to_string());
        }
        if !self.artist.trim().is_empty() {
            parts.push(self.artist.trim().to_string());
        }
        if !self.album.trim().is_empty() {
            parts.push(self.album.trim().to_string());
        }
        parts.join(" / ")
    }
}

#[cfg(target_os = "windows")]
pub use windows::{get_foreground_snapshot, get_now_playing};
#[cfg(target_os = "macos")]
pub use macos::{get_foreground_snapshot, get_now_playing};
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use stub::{get_foreground_snapshot, get_now_playing};

#[cfg(target_os = "windows")]
pub use windows::run_self_test;
#[cfg(target_os = "macos")]
pub use macos::run_self_test;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use stub::run_self_test;

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "unsupported"
    }
}

pub fn make_probe(success: bool, summary: impl Into<String>, detail: impl Into<String>, guidance: Vec<String>) -> PlatformProbeResult {
    PlatformProbeResult {
        success,
        summary: summary.into(),
        detail: detail.into(),
        guidance,
    }
}

pub fn build_self_test_result(
    foreground: PlatformProbeResult,
    window_title: PlatformProbeResult,
    media: PlatformProbeResult,
) -> PlatformSelfTestResult {
    PlatformSelfTestResult {
        platform: platform_name().to_string(),
        foreground,
        window_title,
        media,
    }
}
