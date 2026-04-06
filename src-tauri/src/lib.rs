mod commands;
mod http_client;
mod import_config;
mod models;
mod platform;
mod realtime_reporter;
mod reporter_config;
mod state_store;
#[cfg(desktop)]
mod tray;

use tauri::WindowEvent;

#[cfg(desktop)]
use tauri::Manager;
#[cfg(all(desktop, target_os = "macos"))]
use tauri::RunEvent;

#[cfg(desktop)]
use realtime_reporter::{config_is_ready, ReporterRuntime};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    let builder = builder.manage(ReporterRuntime::new());

    let builder = builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                tray::setup_tray(&app.handle())
                    .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

                let saved_state = state_store::load_app_state(&app.handle())
                    .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

                if saved_state.config.reporter_enabled && config_is_ready(&saved_state.config) {
                    let reporter = app.state::<ReporterRuntime>();
                    let _ = reporter.start(saved_state.config.clone());
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            {
                if window.label() != "main" {
                    return;
                }

                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = tray::hide_main_window(&window.app_handle());
                }
            }
        });

    #[cfg(desktop)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::load_app_state,
        commands::save_app_state,
        commands::parse_imported_integration_config,
        commands::get_client_capabilities,
        commands::hide_to_tray,
        commands::submit_activity_report,
        commands::get_public_activity_feed,
        commands::list_inspiration_entries,
        commands::probe_connectivity,
        commands::create_inspiration_entry,
        commands::upload_inspiration_asset,
        commands::start_realtime_reporter,
        commands::stop_realtime_reporter,
        commands::get_realtime_reporter_snapshot,
        commands::run_platform_self_test,
        commands::discover_existing_reporter_config
    ]);

    #[cfg(mobile)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::load_app_state,
        commands::save_app_state,
        commands::parse_imported_integration_config,
        commands::get_client_capabilities,
        commands::submit_activity_report,
        commands::get_public_activity_feed,
        commands::list_inspiration_entries,
        commands::probe_connectivity,
        commands::create_inspiration_entry,
        commands::upload_inspiration_asset,
        commands::run_platform_self_test,
        commands::discover_existing_reporter_config
    ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {
        #[cfg(all(desktop, target_os = "macos"))]
        if let RunEvent::Reopen { .. } = _event {
            let _ = tray::show_main_window(_app_handle);
        }
    });
}
