use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Runtime, WindowEvent,
};

const TRAY_ID: &str = "envbox-tray";
const WIN_LABEL: &str = "main";

pub fn setup_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray-show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray-quit", "退出 EnvBox", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = app
        .default_window_icon()
        .expect("app icon must be configured")
        .clone();

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("EnvBox")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray-show" => show_main_window(app),
            "tray-quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == WIN_LABEL {
            let _ = window.hide();
            api.prevent_close();
        }
    }
}

fn show_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window(WIN_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
