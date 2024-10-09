use std::sync::Arc;

use tauri::{
    image::Image,
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn setup_windows(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let tray_window = Arc::new(app_handle.get_webview_window("tray-window").unwrap());
    let _ = tray_window.set_visible_on_all_workspaces(true);
    let _ = tray_window.hide();
    let win_clone = tray_window.clone();
    tray_window.on_window_event(move |event| match event {
        WindowEvent::Focused(false) => {
            let _ = win_clone.hide();
        }
        _ => {}
    });

    let app_window = app_handle.get_webview_window("app-window").unwrap();
    let _ = app_window.show();
    TrayIconBuilder::with_id("my-tray")
        .icon(Image::from_path(
            app_handle
                .path()
                .resource_dir()
                .expect("failed to get resource dir")
                .join("icons/icon.ico"),
        )?)
        .on_tray_icon_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
            match event {
                TrayIconEvent::Click { .. } => {
                    let tray_window = app.app_handle().get_webview_window("tray-window").unwrap();
                    let is_visible = tray_window.is_visible().unwrap();
                    if !is_visible {
                        let _ = tray_window
                            .as_ref()
                            .window()
                            .move_window(Position::TrayCenter);
                        let _ = tray_window.as_ref().window().show();
                        let _ = tray_window.set_focus();
                    } else {
                        let _ = tray_window.as_ref().window().hide();
                    }
                }
                _ => {}
            }
        })
        .build(app_handle)?;

    Ok(())
}
