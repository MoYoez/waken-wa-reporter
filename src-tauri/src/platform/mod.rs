#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

use std::time::{SystemTime, UNIX_EPOCH};

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
    /// URL to the cover art image (optional)
    pub cover_url: String,
    pub playback_state: String,
    pub position_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub start_timestamp_ms: Option<i64>,
    pub end_timestamp_ms: Option<i64>,
    pub reported_at_ms: Option<i64>,
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
        let playback_state = if include_media {
            self.playback_state.trim()
        } else {
            ""
        };
        let duration_ms = if include_media {
            self.duration_ms.map(|v| v.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        if title.is_empty() && artist.is_empty() && album.is_empty() && source_app_id.is_empty() {
            return String::new();
        }

        format!(
            "{title}\u{1e}{artist}\u{1e}{album}\u{1e}{source_app_id}\u{1e}{playback_state}\u{1e}{duration_ms}"
        )
    }

    pub fn into_reporting_subset(mut self, include_media: bool, include_play_source: bool) -> Self {
        if !include_media {
            self.title.clear();
            self.artist.clear();
            self.album.clear();
            self.cover_url.clear();
            self.playback_state.clear();
            self.position_ms = None;
            self.duration_ms = None;
            self.start_timestamp_ms = None;
            self.end_timestamp_ms = None;
            self.reported_at_ms = None;
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
        // Include cover URL if available
        if !self.cover_url.trim().is_empty() {
            map.insert(
                "coverDataUrl".into(),
                Value::String(self.cover_url.trim().to_string()),
            );
        }
        let playback_state = self.effective_playback_state();
        if !playback_state.is_empty() {
            map.insert("status".into(), Value::String(playback_state));
        }
        if let Some(position_ms) = self.position_ms.filter(|value| *value >= 0) {
            map.insert("positionMs".into(), Value::Number(position_ms.into()));
        }
        if let Some(duration_ms) = self.duration_ms.filter(|value| *value >= 0) {
            map.insert("durationMs".into(), Value::Number(duration_ms.into()));
        }
        if let Some(reported_at_ms) = self.reported_at_ms.filter(|value| *value > 0) {
            map.insert("reportedAt".into(), Value::Number(reported_at_ms.into()));
        }
        if self.start_timestamp_ms.is_some() || self.end_timestamp_ms.is_some() {
            let mut timestamps = Map::new();
            if let Some(start) = self.start_timestamp_ms.filter(|value| *value > 0) {
                timestamps.insert("start".into(), Value::Number(start.into()));
            }
            if let Some(end) = self.end_timestamp_ms.filter(|value| *value > 0) {
                timestamps.insert("end".into(), Value::Number(end.into()));
            }
            if !timestamps.is_empty() {
                map.insert("timestamps".into(), Value::Object(timestamps));
            }
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

    fn effective_playback_state(&self) -> String {
        let state = self.playback_state.trim();
        if !state.is_empty() {
            return state.to_string();
        }
        if !self.is_empty() {
            return "playing".into();
        }
        String::new()
    }
}

pub fn now_unix_millis_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

pub fn media_timestamps_from_position(
    playback_state: &str,
    position_ms: Option<i64>,
    duration_ms: Option<i64>,
    reported_at_ms: i64,
) -> (Option<i64>, Option<i64>) {
    if playback_state != "playing" {
        return (None, None);
    }
    let Some(position_ms) = position_ms.filter(|value| *value >= 0) else {
        return (None, None);
    };
    let Some(duration_ms) = duration_ms.filter(|value| *value > 0) else {
        return (None, None);
    };

    let clamped_position = position_ms.min(duration_ms);
    (
        Some(reported_at_ms.saturating_sub(clamped_position)),
        Some(reported_at_ms.saturating_add(duration_ms.saturating_sub(clamped_position))),
    )
}

#[allow(unused_imports)]
#[cfg(target_os = "linux")]
pub use linux::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_artwork_for_reporting,
    get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(target_os = "macos")]
pub use macos::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_artwork_for_reporting,
    get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_artwork_for_reporting,
    get_now_playing_for_reporting,
};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
pub use windows::{
    get_foreground_snapshot_for_reporting, get_now_playing, get_now_playing_artwork_for_reporting,
    get_now_playing_for_reporting,
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
