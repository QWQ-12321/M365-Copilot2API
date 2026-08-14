use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Emitter,
};

pub fn setup_tray(app: &AppHandle, port: u16) -> tauri::Result<()> {
    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes)
        .map_err(|e| tauri::Error::Anyhow(e.into()))?;

    let show_i = MenuItem::with_id(app, "show", "打开管理界面", true, None::<&str>)?;
    let open_browser_i = MenuItem::with_id(app, "browser", "在浏览器中打开", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &open_browser_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip(format!("M365 Copilot2API — 端口 {}", port))
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "browser" => {
                let _ = app.emit("open-browser", port);
            }
            "quit" => {
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
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
