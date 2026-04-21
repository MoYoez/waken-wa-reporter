use chrono::DateTime;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::backend_locale::BackendLocale;

use super::{
    feed::PublicActivityItem,
    messages::{discord_ipc_unavailable, format_error},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DiscordPresencePayload {
    pub(super) details: String,
    pub(super) state: Option<String>,
    pub(super) started_at_millis: Option<i64>,
    pub(super) summary: String,
}

pub(super) fn map_activity_to_presence(item: &PublicActivityItem) -> DiscordPresencePayload {
    let details_source = item
        .status_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback_details(item));
    let details = normalize_presence_line(&details_source);

    let state = item
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_presence_line);
    let started_at_millis = item.started_at.as_deref().and_then(parse_started_at_millis);
    let summary = match state.as_deref() {
        Some(device) => format!("{details} · {device}"),
        None => details.clone(),
    };

    DiscordPresencePayload {
        details,
        state,
        started_at_millis,
        summary,
    }
}

fn fallback_details(item: &PublicActivityItem) -> String {
    let process_name = item
        .process_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let process_title = item
        .process_title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (process_title, process_name) {
        (Some(title), Some(name)) => format!("{title} | {name}"),
        (Some(title), None) => title.to_string(),
        (None, Some(name)) => name.to_string(),
        (None, None) => "Waken-Wa".to_string(),
    }
}

pub(super) fn apply_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    payload: &DiscordPresencePayload,
    locale: BackendLocale,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        let mut activity_payload = activity::Activity::new().details(payload.details.clone());
        if let Some(state) = payload.state.as_deref() {
            activity_payload = activity_payload.state(state.to_string());
        }
        if let Some(started_at) = payload.started_at_millis {
            activity_payload =
                activity_payload.timestamps(activity::Timestamps::new().start(started_at));
        }
        client.set_activity(activity_payload).map_err(|error| {
            format_error(
                locale,
                "更新 Discord 状态失败",
                "Failed to update Discord presence",
                error,
            )
        })
    })
}

pub(super) fn clear_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        client.clear_activity().map_err(|error| {
            format_error(
                locale,
                "清空 Discord 状态失败",
                "Failed to clear Discord presence",
                error,
            )
        })
    })
}

fn with_discord_client<F>(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
    mut action: F,
) -> Result<(), String>
where
    F: FnMut(&mut DiscordIpcClient) -> Result<(), String>,
{
    for _ in 0..2 {
        if client_slot.is_none() {
            let mut client = DiscordIpcClient::new(application_id);
            client.connect().map_err(|error| {
                format_error(
                    locale,
                    "连接 Discord IPC 失败",
                    "Failed to connect to Discord IPC",
                    error,
                )
            })?;
            *client_slot = Some(client);
        }

        let Some(client) = client_slot.as_mut() else {
            continue;
        };

        match action(client) {
            Ok(()) => return Ok(()),
            Err(_) => {
                *client_slot = None;
            }
        }
    }

    Err(discord_ipc_unavailable(locale))
}

fn normalize_presence_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_started_at_millis(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp_millis())
}
