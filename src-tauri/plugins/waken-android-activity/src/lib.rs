use tauri::{plugin::TauriPlugin, Runtime};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.waken_wa_reporter.plugin.activity";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("waken-android-activity")
        .setup(|_app, api| {
            #[cfg(target_os = "android")]
            let _handle =
                api.register_android_plugin(PLUGIN_IDENTIFIER, "WakenAndroidActivityPlugin")?;

            Ok(())
        })
        .build()
}
