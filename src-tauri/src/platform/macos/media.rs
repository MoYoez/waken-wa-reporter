use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::Deserialize;

use crate::platform::{media_timestamps_from_position, now_unix_millis_i64, MediaInfo};

use super::command::{command_output_with_timeout, CommandError};

const NOWPLAYING_CLI: &str = "nowplaying-cli";
const NOWPLAYING_CLI_FALLBACK_PATHS: [&str; 2] = [
    "/opt/homebrew/bin/nowplaying-cli",
    "/usr/local/bin/nowplaying-cli",
];

enum NowPlayingCliError {
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl NowPlayingCliError {
    fn into_user_message(self) -> String {
        match self {
            Self::NotFound { path, attempted } => {
                format!(
                    "调用 nowplaying-cli 失败：未在全局环境或 Homebrew 常见路径中找到可执行文件。已尝试：{}。PATH={path}",
                    attempted.join(", ")
                )
            }
            Self::TimedOut => "调用 nowplaying-cli 超时（>1500ms）。".into(),
            Self::Failed(detail) => format!("nowplaying-cli 返回失败：{detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoTitle")]
    title: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoArtist")]
    artist: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoAlbum")]
    album: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoClientBundleIdentifier")]
    client_bundle_identifier: Option<String>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoArtworkData",
        alias = "artworkData"
    )]
    artwork_data: Option<String>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoArtworkMIMEType",
        alias = "artworkMimeType"
    )]
    artwork_mime_type: Option<String>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoElapsedTime",
        alias = "elapsedTime"
    )]
    elapsed_time: Option<f64>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoDuration", alias = "duration")]
    duration: Option<f64>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoPlaybackRate",
        alias = "playbackRate"
    )]
    playback_rate: Option<f64>,
}

pub(super) fn get_now_playing() -> Result<MediaInfo, String> {
    let media = match get_now_playing_via_nowplaying_cli() {
        Ok(media) => media,
        Err(NowPlayingCliError::TimedOut) => return Ok(MediaInfo::default()),
        Err(error) => return Err(error.into_user_message()),
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, NowPlayingCliError> {
    let attempted = std::iter::once(NOWPLAYING_CLI)
        .chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let output = {
        let mut resolved = None;
        for candidate in
            std::iter::once(NOWPLAYING_CLI).chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        {
            match command_output_with_timeout(candidate, &["get-raw"]) {
                Ok(output) => {
                    resolved = Some(output);
                    break;
                }
                Err(CommandError::NotFound) => {}
                Err(CommandError::TimedOut) => return Err(NowPlayingCliError::TimedOut),
                Err(CommandError::Other(detail)) => return Err(NowPlayingCliError::Failed(detail)),
            }
        }
        resolved
    }
    .ok_or_else(|| NowPlayingCliError::NotFound {
        path: std::env::var("PATH").unwrap_or_default(),
        attempted,
    })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
            || combined.contains("null")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("未知错误");
        return Err(NowPlayingCliError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalize = |value: String| {
        if value.eq_ignore_ascii_case("null") {
            String::new()
        } else {
            value.trim().to_string()
        }
    };

    let raw: RawNowPlayingInfo = serde_json::from_str(&stdout)
        .map_err(|error| NowPlayingCliError::Failed(format!("解析 get-raw 输出失败：{error}")))?;

    let title = raw.title.map(normalize).unwrap_or_default();
    let artist = raw.artist.map(normalize).unwrap_or_default();
    let album = raw.album.map(normalize).unwrap_or_default();
    let source_app_id = raw
        .client_bundle_identifier
        .map(normalize)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| NOWPLAYING_CLI.to_string());

    let cover_url = decode_artwork_to_data_url(
        raw.artwork_data.as_deref(),
        raw.artwork_mime_type.as_deref(),
    );
    let playback_state = normalize_playback_state(raw.playback_rate);
    let position_ms = seconds_to_ms(raw.elapsed_time);
    let duration_ms = seconds_to_ms(raw.duration);
    let reported_at_ms = now_unix_millis_i64();
    let (start_timestamp_ms, end_timestamp_ms) =
        media_timestamps_from_position(&playback_state, position_ms, duration_ms, reported_at_ms);

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        cover_url,
        playback_state,
        position_ms,
        duration_ms,
        start_timestamp_ms,
        end_timestamp_ms,
        reported_at_ms: Some(reported_at_ms),
    })
}

fn normalize_playback_state(playback_rate: Option<f64>) -> String {
    match playback_rate {
        Some(rate) if rate > 0.0 => "playing".into(),
        Some(_) => "paused".into(),
        None => String::new(),
    }
}

fn seconds_to_ms(value: Option<f64>) -> Option<i64> {
    let seconds = value?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some((seconds * 1000.0).round() as i64)
}

fn decode_artwork_to_data_url(artwork_data: Option<&str>, mime_type: Option<&str>) -> String {
    let data = match artwork_data.and_then(decode_base64_image_payload) {
        Some(bytes) => bytes,
        None => return String::new(),
    };

    let content_type = mime_type
        .and_then(|v| v.trim().trim_start_matches("data:").split(';').next())
        .filter(|v| v.starts_with("image/"))
        .map(str::to_string)
        .or_else(|| detect_image_content_type(&data))
        .unwrap_or_else(|| "image/jpeg".to_string());

    let encoded = BASE64_STANDARD.encode(&data);
    format!("data:{content_type};base64,{encoded}")
}

fn decode_base64_image_payload(value: &str) -> Option<Vec<u8>> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return None;
    }
    let encoded = trimmed
        .split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(trimmed);
    BASE64_STANDARD
        .decode(encoded.trim())
        .ok()
        .filter(|bytes| !bytes.is_empty())
}

fn detect_image_content_type(bytes: &[u8]) -> Option<String> {
    let s = if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        "image/png"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        return None;
    };
    Some(s.to_string())
}
