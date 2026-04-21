use crate::platform::MediaInfo;

use super::command::{command_output_with_timeout, EmptyFallback};

pub(super) fn get_now_playing() -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}\n{{playerName}}",
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

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}
