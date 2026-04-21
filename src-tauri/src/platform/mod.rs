#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

use serde_json::{Map, Value};

use crate::models::{LocalizedTextEntry, PlatformProbeResult, PlatformSelfTestResult};

#[derive(Clone, Debug, Default)]
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
        self.title.trim().is_empty()
            && self.artist.trim().is_empty()
            && self.album.trim().is_empty()
    }

    pub fn has_play_source(&self) -> bool {
        !self.source_app_id.trim().is_empty()
    }

    pub fn signature_for_reporting(
        &self,
        include_media: bool,
        include_play_source: bool,
    ) -> String {
        let title = if include_media { self.title.trim() } else { "" };
        let artist = if include_media {
            self.artist.trim()
        } else {
            ""
        };
        let album = if include_media { self.album.trim() } else { "" };
        let source_app_id = if include_play_source {
            self.source_app_id.trim()
        } else {
            ""
        };

        if title.is_empty() && artist.is_empty() && album.is_empty() && source_app_id.is_empty() {
            return String::new();
        }

        format!("{title}\u{1e}{artist}\u{1e}{album}\u{1e}{source_app_id}")
    }

    pub fn into_reporting_subset(mut self, include_media: bool, include_play_source: bool) -> Self {
        if !include_media {
            self.title.clear();
            self.artist.clear();
            self.album.clear();
        }
        if !include_play_source {
            self.source_app_id.clear();
        }

        if self.is_empty() && !self.has_play_source() {
            return Self::default();
        }

        self
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
            map.insert(
                "artist".into(),
                Value::String(self.artist.trim().to_string()),
            );
            map.insert(
                "singer".into(),
                Value::String(self.artist.trim().to_string()),
            );
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

#[allow(unused_imports)]
#[cfg(target_os = "linux")]
pub use linux::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(target_os = "macos")]
pub use macos::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
pub use windows::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_for_reporting,
};

#[cfg(target_os = "linux")]
pub use linux::run_self_test;
#[cfg(target_os = "macos")]
pub use macos::run_self_test;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::run_self_test;
#[cfg(target_os = "windows")]
pub use windows::run_self_test;

#[cfg(target_os = "linux")]
pub use linux::request_accessibility_permission;
#[cfg(target_os = "macos")]
pub use macos::request_accessibility_permission;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::request_accessibility_permission;
#[cfg(target_os = "windows")]
pub use windows::request_accessibility_permission;

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unsupported"
    }
}

pub fn make_probe(
    success: bool,
    summary: ProbeTextSpec,
    detail: ProbeTextSpec,
    guidance: Vec<ProbeTextSpec>,
) -> PlatformProbeResult {
    let ProbeTextSpec {
        key: summary_key,
        params: summary_params,
        fallback: summary_fallback,
    } = summary;
    let ProbeTextSpec {
        key: detail_key,
        params: detail_params,
        fallback: detail_fallback,
    } = detail;

    PlatformProbeResult {
        success,
        summary: summary_fallback,
        detail: detail_fallback,
        guidance: guidance
            .iter()
            .map(|entry| entry.fallback.clone())
            .collect(),
        summary_key: summary_key.map(str::to_string),
        summary_params,
        detail_key: detail_key.map(str::to_string),
        detail_params,
        guidance_entries: guidance
            .into_iter()
            .map(|entry| LocalizedTextEntry {
                text: entry.fallback,
                key: entry.key.map(str::to_string),
                params: entry.params,
            })
            .collect(),
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

pub struct ProbeTextSpec {
    key: Option<&'static str>,
    params: Option<Value>,
    fallback: String,
}

pub fn localized_text(
    key: &'static str,
    params: Option<Value>,
    fallback: impl Into<String>,
) -> ProbeTextSpec {
    ProbeTextSpec {
        key: Some(key),
        params,
        fallback: fallback.into(),
    }
}
