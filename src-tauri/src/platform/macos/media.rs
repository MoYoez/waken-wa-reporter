use serde::Deserialize;

use crate::platform::MediaInfo;

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

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
    })
}
