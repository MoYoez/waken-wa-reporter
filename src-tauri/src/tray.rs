use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::state_store;

const MENU_ID_SHOW: &str = "tray_show";
const MENU_ID_HIDE: &str = "tray_hide";
const MENU_ID_QUIT: &str = "tray_quit";

enum TrayText {
    MainWindowNotFound,
    Show,
    Hide,
    Quit,
    CreateMenuFailed,
    LoadIconFailed,
    CreateTrayFailed,
}

fn current_locale(app: &AppHandle) -> String {
    state_store::load_app_state(app)
        .map(|state| state.locale)
        .unwrap_or_default()
}

fn tray_text(locale: &str, key: TrayText) -> &'static str {
    let english = locale.trim().to_ascii_lowercase().starts_with("en");

    match (english, key) {
        (true, TrayText::MainWindowNotFound) => "Main window not found.",
        (true, TrayText::Show) => "Open main window",
        (true, TrayText::Hide) => "Hide to background",
        (true, TrayText::Quit) => "Quit app",
        (true, TrayText::CreateMenuFailed) => "Failed to create the tray menu",
        (true, TrayText::LoadIconFailed) => "Failed to load the tray icon",
        (true, TrayText::CreateTrayFailed) => "Failed to create the system tray",
        (false, TrayText::MainWindowNotFound) => "未找到主窗口。",
        (false, TrayText::Show) => "打开主界面",
        (false, TrayText::Hide) => "隐藏到后台",
        (false, TrayText::Quit) => "退出应用",
        (false, TrayText::CreateMenuFailed) => "创建托盘菜单失败",
        (false, TrayText::LoadIconFailed) => "加载托盘图标失败",
        (false, TrayText::CreateTrayFailed) => "创建系统托盘失败",
    }
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let locale = current_locale(app);
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tray_text(&locale, TrayText::MainWindowNotFound).to_string())?;

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    let _ = window.set_skip_taskbar(false);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let locale = current_locale(app);
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tray_text(&locale, TrayText::MainWindowNotFound).to_string())?;
    let _ = window.hide();
    let _ = window.set_skip_taskbar(true);

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_dock_visibility(false);
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    Ok(())
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let locale = current_locale(app);
    let show_item = MenuItem::with_id(
        app,
        MENU_ID_SHOW,
        tray_text(&locale, TrayText::Show),
        true,
        None::<&str>,
    )
    .map_err(|error| {
        format!(
            "{}: {error}",
            tray_text(&locale, TrayText::CreateMenuFailed)
        )
    })?;
    let hide_item = MenuItem::with_id(
        app,
        MENU_ID_HIDE,
        tray_text(&locale, TrayText::Hide),
        true,
        None::<&str>,
    )
    .map_err(|error| {
        format!(
            "{}: {error}",
            tray_text(&locale, TrayText::CreateMenuFailed)
        )
    })?;
    let quit_item = MenuItem::with_id(
        app,
        MENU_ID_QUIT,
        tray_text(&locale, TrayText::Quit),
        true,
        None::<&str>,
    )
    .map_err(|error| {
        format!(
            "{}: {error}",
            tray_text(&locale, TrayText::CreateMenuFailed)
        )
    })?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item]).map_err(|error| {
        format!(
            "{}: {error}",
            tray_text(&locale, TrayText::CreateMenuFailed)
        )
    })?;
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
        .map_err(|error| format!("{}: {error}", tray_text(&locale, TrayText::LoadIconFailed)))?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_ID_SHOW => {
                let _ = show_main_window(app);
            }
            MENU_ID_HIDE => {
                let _ = hide_main_window(app);
            }
            MENU_ID_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| {
            format!(
                "{}: {error}",
                tray_text(&locale, TrayText::CreateTrayFailed)
            )
        })?;

    Ok(())
}
