use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

use crate::platform::{media_timestamps_from_position, now_unix_millis_i64, MediaInfo};

use super::command::{command_output_with_timeout, EmptyFallback};

const ARTWORK_DOWNLOAD_TIMEOUT_MS: u64 = 5000;

pub(super) fn get_now_playing() -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}\n{{playerName}}\n{{mpris:artUrl}}\n{{status}}\n{{position}}\n{{mpris:length}}",
        ],
    )
    .map_err(|error| format!("调用 playerctl 失败：{error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(MediaInfo::default());
        }
        return Err(format!(
            "读取媒体信息失败：{}",
            stderr.trim().if_empty("playerctl 返回失败")
        ));
    }

    let mut lines = stdout.lines().map(str::trim);
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();
    let source_app_id = lines.next().unwrap_or_default().to_string();
    let cover_url_raw = lines.next().unwrap_or_default().to_string();
    let playback_state = normalize_playback_state(lines.next().unwrap_or_default());
    let position_ms = parse_position_seconds_to_ms(lines.next().unwrap_or_default());
    let duration_ms = parse_mpris_length_to_ms(lines.next().unwrap_or_default());
    let reported_at_ms = now_unix_millis_i64();
    let (start_timestamp_ms, end_timestamp_ms) =
        media_timestamps_from_position(&playback_state, position_ms, duration_ms, reported_at_ms);

    // Convert HTTP artUrl to base64 data URL if needed
    let cover_url = resolve_artwork_data_url(&cover_url_raw);

    let media = MediaInfo {
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
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

fn normalize_playback_state(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "playing" | "play" => "playing".into(),
        "paused" | "pause" => "paused".into(),
        "stopped" | "stop" => "stopped".into(),
        _ => String::new(),
    }
}

fn parse_position_seconds_to_ms(value: &str) -> Option<i64> {
    let seconds = value.trim().parse::<f64>().ok()?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some((seconds * 1000.0).round() as i64)
}

fn parse_mpris_length_to_ms(value: &str) -> Option<i64> {
    let micros = value.trim().parse::<i64>().ok()?;
    if micros < 0 {
        return None;
    }
    Some(micros / 1000)
}

/// Resolve artwork URL to base64 data URL.
/// If `input` is already a data URL, return as-is.
/// If `input` is an HTTP(S) URL, download and encode as data URL.
/// Otherwise return empty string.
fn resolve_artwork_data_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Already a data URL — return as-is
    if trimmed.starts_with("data:") {
        return trimmed.to_string();
    }

    // Must be http:// or https:// to download
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return String::new();
    }

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(
            ARTWORK_DOWNLOAD_TIMEOUT_MS,
        ))
        .build()
    {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let response = match client.get(trimmed).send() {
        Ok(resp) => resp,
        Err(_) => return String::new(),
    };

    if !response.status().is_success() {
        return String::new();
    }

    let header_content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().trim_start_matches("data:").split(';').next())
        .filter(|v| v.starts_with("image/"))
        .map(str::to_string);

    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(_) => return String::new(),
    };

    if bytes.is_empty() {
        return String::new();
    }

    let content_type = header_content_type.unwrap_or_else(|| detect_image_content_type(&bytes));

    let encoded = BASE64_STANDARD.encode(&bytes);
    format!("data:{content_type};base64,{encoded}")
}

fn detect_image_content_type(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".to_string()
    } else if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        "image/png".to_string()
    } else if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        "image/gif".to_string()
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp".to_string()
    } else {
        "image/jpeg".to_string()
    }
}
