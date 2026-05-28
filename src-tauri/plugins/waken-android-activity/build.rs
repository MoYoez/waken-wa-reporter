const COMMANDS: &[&str] = &[
    "start_foreground_service",
    "stop_foreground_service",
    "register_listener",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
