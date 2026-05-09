use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::Deserialize;

use crate::platform::{
    image_bytes_to_png_data_url, media_timestamps_from_position, now_unix_millis_i64, MediaInfo,
};

use super::{
    bridge::{read_bundle_app_icon_data_url, read_bundle_app_name},
    command::{command_output_with_timeout, CommandError},
};

const PERL_BINARY: &str = "/usr/bin/perl";
const ADAPTER_SCRIPT_NAME: &str = "mediaremote-adapter.pl";
const ADAPTER_FRAMEWORK_NAME: &str = "MediaRemoteAdapter.framework";
const ADAPTER_TIMEOUT_ERROR_HINT: &str =
    "请先运行 `pnpm tauri dev` / `pnpm tauri build`，或手动执行 `pnpm prepare:mediaremote-adapter`。";

enum MediaRemoteAdapterError {
    ResourceMissing(String),
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl MediaRemoteAdapterError {
    fn into_user_message(self) -> String {
        match self {
            Self::ResourceMissing(path) => format!(
                "缺少 macOS mediaremote-adapter 资源：{path}。{ADAPTER_TIMEOUT_ERROR_HINT}"
            ),
            Self::NotFound { path, attempted } => format!(
                "调用 mediaremote-adapter 失败：未在资源目录中找到脚本或框架。已尝试：{}。PATH={path}",
                attempted.join(", ")
            ),
            Self::TimedOut => "调用 mediaremote-adapter 超时（>5000ms）。".into(),
            Self::Failed(detail) => format!("mediaremote-adapter 返回失败：{detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "bundleIdentifier")]
    bundle_identifier: Option<String>,
    #[serde(rename = "parentApplicationBundleIdentifier")]
    parent_application_bundle_identifier: Option<String>,
    #[serde(rename = "playing")]
    playing: Option<bool>,
    #[serde(rename = "title")]
    title: Option<String>,
    #[serde(rename = "artist")]
    artist: Option<String>,
    #[serde(rename = "album")]
    album: Option<String>,
    #[serde(rename = "durationMicros")]
    duration_micros: Option<i64>,
    #[serde(rename = "elapsedTimeMicros")]
    elapsed_time_micros: Option<i64>,
    #[serde(rename = "elapsedTimeNowMicros")]
    elapsed_time_now_micros: Option<i64>,
    #[serde(rename = "timestampEpochMicros")]
    timestamp_epoch_micros: Option<i64>,
    #[serde(rename = "playbackRate")]
    playback_rate: Option<f64>,
    #[serde(rename = "artworkData")]
    artwork_data: Option<String>,
    #[serde(rename = "artworkMimeType")]
    artwork_mime_type: Option<String>,
}

pub(super) fn get_now_playing() -> Result<MediaInfo, String> {
    read_now_playing_with_adapter(false, false)
}

pub(super) fn get_now_playing_with_artwork(include_source_icon: bool) -> Result<MediaInfo, String> {
    read_now_playing_with_adapter(true, include_source_icon)
}

fn read_now_playing_with_adapter(
    include_artwork: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, String> {
    let media = match get_now_playing_via_mediaremote_adapter(include_artwork, include_source_icon)
    {
        Ok(media) => media,
        Err(MediaRemoteAdapterError::TimedOut) => {
            return Err(MediaRemoteAdapterError::TimedOut.into_user_message())
        }
        Err(error) => return Err(error.into_user_message()),
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

fn get_now_playing_via_mediaremote_adapter(
    include_artwork: bool,
    include_source_icon: bool,
) -> Result<MediaInfo, MediaRemoteAdapterError> {
    let (script_path, framework_path) = resolve_adapter_paths()?;
    let attempted = vec![
        script_path.to_string_lossy().to_string(),
        framework_path.to_string_lossy().to_string(),
    ];

    let mut args = vec![
        script_path.to_string_lossy().to_string(),
        framework_path.to_string_lossy().to_string(),
        "get".to_string(),
        "--now".to_string(),
        "--micros".to_string(),
    ];
    if !include_artwork {
        args.push("--no-artwork".to_string());
    }
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    let output = match command_output_with_timeout(PERL_BINARY, &arg_refs) {
        Ok(output) => output,
        Err(CommandError::NotFound) => {
            let path = std::env::var("PATH").unwrap_or_default();
            return Err(MediaRemoteAdapterError::NotFound { path, attempted });
        }
        Err(CommandError::TimedOut) => return Err(MediaRemoteAdapterError::TimedOut),
        Err(CommandError::Other(detail)) => return Err(MediaRemoteAdapterError::Failed(detail)),
    };

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("null")
            || combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("未知错误");
        return Err(MediaRemoteAdapterError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(MediaInfo::default());
    }

    let raw: Option<RawNowPlayingInfo> = serde_json::from_str(trimmed)
        .map_err(|error| MediaRemoteAdapterError::Failed(format!("解析 get 输出失败：{error}")))?;
    let Some(raw) = raw else {
        return Ok(MediaInfo::default());
    };

    let source_app_id = resolve_source_app_id(&raw);
    let source_app_name = resolve_source_app_name(&source_app_id);
    let source_icon_url = if include_source_icon {
        resolve_source_icon_data_url(&raw)
    } else {
        String::new()
    };
    let playback_state = normalize_playback_state(raw.playing, raw.playback_rate);
    let position_ms = resolve_position_ms(&raw);
    let duration_ms = micros_to_ms(raw.duration_micros);
    let reported_at_ms = now_unix_millis_i64();
    let (start_timestamp_ms, end_timestamp_ms) =
        media_timestamps_from_position(&playback_state, position_ms, duration_ms, reported_at_ms);

    let cover_url = if include_artwork {
        decode_artwork_to_data_url(
            raw.artwork_data.as_deref(),
            raw.artwork_mime_type.as_deref(),
        )
    } else {
        String::new()
    };
    let title = normalize_text(raw.title);
    let artist = normalize_text(raw.artist);
    let album = normalize_text(raw.album);

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        source_app_name,
        cover_url,
        source_icon_url,
        playback_state,
        position_ms,
        duration_ms,
        start_timestamp_ms,
        end_timestamp_ms,
        reported_at_ms: Some(reported_at_ms),
        genre: String::new(),
    })
}

fn resolve_adapter_paths(
) -> Result<(std::path::PathBuf, std::path::PathBuf), MediaRemoteAdapterError> {
    let mut candidates = Vec::new();

    if let Some(root) = crate::platform::macos_mediaremote_adapter_root() {
        candidates.push(root.clone());
    }

    let compiled_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("mediaremote-adapter");
    if !candidates
        .iter()
        .any(|candidate| candidate == &compiled_root)
    {
        candidates.push(compiled_root);
    }

    let root = candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            MediaRemoteAdapterError::ResourceMissing("resources/mediaremote-adapter".to_string())
        })?;

    let script_path = root.join(ADAPTER_SCRIPT_NAME);
    let framework_path = root.join(ADAPTER_FRAMEWORK_NAME);
    let framework_binary = framework_path.join("MediaRemoteAdapter");
    if !script_path.exists() || !framework_binary.exists() {
        return Err(MediaRemoteAdapterError::NotFound {
            path: std::env::var("PATH").unwrap_or_default(),
            attempted: vec![
                script_path.to_string_lossy().to_string(),
                framework_path.to_string_lossy().to_string(),
            ],
        });
    }

    Ok((script_path, framework_path))
}

fn normalize_text(value: Option<String>) -> String {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
        .unwrap_or_default()
}

fn normalize_optional_str(value: Option<&str>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
}

fn resolve_source_app_id(raw: &RawNowPlayingInfo) -> String {
    normalize_optional_str(raw.bundle_identifier.as_deref())
        .or_else(|| normalize_optional_str(raw.parent_application_bundle_identifier.as_deref()))
        .unwrap_or_default()
}

fn resolve_source_app_name(source_app_id: &str) -> String {
    read_bundle_app_name(source_app_id).unwrap_or_default()
}

fn normalize_playback_state(playing: Option<bool>, playback_rate: Option<f64>) -> String {
    match playing {
        Some(true) => "playing".into(),
        Some(false) => "paused".into(),
        None => match playback_rate {
            Some(rate) if rate > 0.0 => "playing".into(),
            Some(_) => "paused".into(),
            None => String::new(),
        },
    }
}

fn resolve_position_ms(raw: &RawNowPlayingInfo) -> Option<i64> {
    if let Some(value) = micros_to_ms(raw.elapsed_time_now_micros) {
        return Some(value);
    }

    let Some(elapsed_micros) = raw.elapsed_time_micros else {
        return None;
    };

    if raw.playing != Some(true) {
        return micros_to_ms(Some(elapsed_micros));
    }

    let Some(timestamp_epoch_micros) = raw.timestamp_epoch_micros else {
        return micros_to_ms(Some(elapsed_micros));
    };

    let now_micros = now_unix_micros_i64();
    let playback_rate = raw
        .playback_rate
        .filter(|value| value.is_finite())
        .unwrap_or(1.0)
        .max(0.0);
    let elapsed_seconds = elapsed_micros as f64 / 1_000_000.0;
    let elapsed_since_timestamp_seconds =
        now_micros.saturating_sub(timestamp_epoch_micros).max(0) as f64 / 1_000_000.0;
    Some(
        ((elapsed_seconds + elapsed_since_timestamp_seconds * playback_rate) * 1000.0).round()
            as i64,
    )
}

fn micros_to_ms(value: Option<i64>) -> Option<i64> {
    let micros = value?;
    if micros < 0 {
        return None;
    }
    Some(micros / 1000)
}

fn now_unix_micros_i64() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_micros()).ok())
        .unwrap_or(0)
}

fn resolve_source_icon_data_url(raw: &RawNowPlayingInfo) -> String {
    let candidates = [
        raw.bundle_identifier.as_deref(),
        raw.parent_application_bundle_identifier.as_deref(),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(bundle_id) = normalize_optional_str(Some(candidate)) {
            if let Some(icon) = read_bundle_app_icon_data_url(&bundle_id) {
                return icon;
            }
        }
    }

    String::new()
}

fn decode_artwork_to_data_url(artwork_data: Option<&str>, mime_type: Option<&str>) -> String {
    let data = match artwork_data.and_then(decode_base64_image_payload) {
        Some(bytes) => bytes,
        None => return String::new(),
    };

    let _ = mime_type;
    image_bytes_to_png_data_url(&data).unwrap_or_default()
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
