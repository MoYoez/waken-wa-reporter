use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const MENU_ID_SHOW: &str = "tray_show";
const MENU_ID_HIDE: &str = "tray_hide";
const MENU_ID_QUIT: &str = "tray_quit";

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口。".to_string())?;

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_dock_visibility(false);
    }

    let _ = window.set_skip_taskbar(false);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口。".to_string())?;
    let _ = window.hide();
    let _ = window.set_skip_taskbar(true);
    Ok(())
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let show_item = MenuItem::with_id(app, MENU_ID_SHOW, "打开主界面", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let hide_item = MenuItem::with_id(app, MENU_ID_HIDE, "隐藏到后台", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let quit_item = MenuItem::with_id(app, MENU_ID_QUIT, "退出应用", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
        .map_err(|error| format!("加载托盘图标失败：{error}"))?;

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
        .map_err(|error| format!("创建系统托盘失败：{error}"))?;

    Ok(())
}
