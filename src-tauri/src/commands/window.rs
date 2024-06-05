use tauri::Manager;

#[tauri::command]
pub async fn open_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let window = app_handle.get_webview_window("app-window");
    if window.is_none() {
        println!("No window found");
        let mut window = tauri::WebviewWindowBuilder::from_config(
            &app_handle,
            &app_handle.config().app.windows.get(1).unwrap().clone(),
        )
        .unwrap()
        .build()
        .expect("Failed to create window");
        window.navigate(window.url().unwrap().join("/tray").unwrap());
    } else {
        println!("found window");
    }

    Ok(())
}
