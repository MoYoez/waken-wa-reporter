use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

use crate::platform::{media_timestamps_from_position, now_unix_millis_i64, MediaInfo};

use super::app_icon::read_source_app_icon_data_url;
use super::command::{command_output_with_timeout, EmptyFallback};

const ARTWORK_DOWNLOAD_TIMEOUT_MS: u64 = 5000;
const MPRIS_BUS_PREFIX: &str = "org.mpris.MediaPlayer2.";

pub(super) fn get_now_playing(include_source_icon: bool) -> Result<MediaInfo, String> {
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
    let source_app_name = source_app_id.clone();
    let cover_url_raw = lines.next().unwrap_or_default().to_string();
    let playback_state = normalize_playback_state(lines.next().unwrap_or_default());
    let position_ms = parse_microseconds_to_ms(lines.next().unwrap_or_default());
    let duration_ms = parse_mpris_length_to_ms(lines.next().unwrap_or_default());
    let reported_at_ms = now_unix_millis_i64();
    let (start_timestamp_ms, end_timestamp_ms) =
        media_timestamps_from_position(&playback_state, position_ms, duration_ms, reported_at_ms);

    // Convert HTTP artUrl to base64 data URL if needed
    let cover_url = resolve_artwork_data_url(&cover_url_raw);
    let desktop_entry = if include_source_icon {
        let player_instance = read_player_instance().unwrap_or_default();
        read_mpris_root_string_property(&player_instance, "DesktopEntry")
            .or_else(|| read_mpris_root_string_property(&source_app_id, "DesktopEntry"))
    } else {
        None
    };
    let source_icon_url = if include_source_icon {
        read_source_app_icon_data_url(&source_app_id, desktop_entry.as_deref())
    } else {
        String::new()
    };

    let media = MediaInfo {
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
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

fn read_player_instance() -> Option<String> {
    let output =
        command_output_with_timeout("playerctl", &["metadata", "--format", "{{playerInstance}}"])
            .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout);
    let value = value.lines().next().unwrap_or_default().trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn read_mpris_root_string_property(player_instance: &str, property: &str) -> Option<String> {
    let player_instance = player_instance.trim();
    if player_instance.is_empty() {
        return None;
    }

    let destination = if player_instance.starts_with(MPRIS_BUS_PREFIX) {
        player_instance.to_string()
    } else {
        format!("{MPRIS_BUS_PREFIX}{player_instance}")
    };
    let output = command_output_with_timeout(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            &destination,
            "--object-path",
            "/org/mpris/MediaPlayer2",
            "--method",
            "org.freedesktop.DBus.Properties.Get",
            "org.mpris.MediaPlayer2",
            property,
        ],
    )
    .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_gdbus_string_variant(&String::from_utf8_lossy(&output.stdout))
}

fn parse_gdbus_string_variant(output: &str) -> Option<String> {
    let start = output.find('\'')?;
    let mut value = String::new();
    let mut escaped = false;

    for ch in output[start + 1..].chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '\'' => break,
            _ => value.push(ch),
        }
    }

    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn normalize_playback_state(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "playing" | "play" => "playing".into(),
        "paused" | "pause" => "paused".into(),
        "stopped" | "stop" => "stopped".into(),
        _ => String::new(),
    }
}

fn parse_microseconds_to_ms(value: &str) -> Option<i64> {
    let micros = value.trim().parse::<i64>().ok()?;
    if micros < 0 {
        return None;
    }
    Some(micros / 1000)
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
        .map(str::trim)
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
    } else if bytes.len() >= 4
        && bytes[0] == 0x00
        && bytes[1] == 0x00
        && (bytes[2] == 0x01 || bytes[2] == 0x02)
        && bytes[3] == 0x00
    {
        "image/x-icon".to_string()
    } else if looks_like_svg(bytes) {
        "image/svg+xml".to_string()
    } else {
        "image/jpeg".to_string()
    }
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let sample = bytes.len().min(512);
    let text = String::from_utf8_lossy(&bytes[..sample]).to_ascii_lowercase();
    text.contains("<svg")
}
