use tauri::{AppHandle, State};

use super::helpers::{discord_start_error_code, reporter_start_error_code};
use crate::{
    backend_locale::load_locale,
    discord_presence::DiscordPresenceRuntime,
    models::{ApiResult, ClientConfig, DiscordPresenceSnapshot, RealtimeReporterSnapshot},
    realtime_reporter::{snapshot_result, ReporterRuntime},
    tray,
};

pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    tray::hide_main_window(&app)
}

pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    match reporter.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            reporter_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

pub fn stop_realtime_reporter(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    Ok(ApiResult::success(200, reporter.stop()))
}

pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    Ok(snapshot_result(&reporter))
}

pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    match discord_presence_runtime.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            discord_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

pub fn stop_discord_presence_sync(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.stop()))
}

pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.snapshot()))
}
