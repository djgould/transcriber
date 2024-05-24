// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod device_listener;
mod media;
mod recorder;
mod summarize;
mod transcribe;
mod utils;

use coreaudio::audio_unit::macos_helpers::get_default_device_id;
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::download::check_latest_version;
use ffmpeg_sidecar::download::download_ffmpeg_package;
use ffmpeg_sidecar::download::ffmpeg_download_url;
use ffmpeg_sidecar::download::unpack_ffmpeg;
use ffmpeg_sidecar::error::Result as FfmpegResult;
use ffmpeg_sidecar::paths::sidecar_dir;
use ffmpeg_sidecar::version::ffmpeg_version;
use mac_notification_sys::get_bundle_identifier_or_default;
use mac_notification_sys::send_notification;
use mac_notification_sys::set_application;
use migration::Migrator;
use migration::MigratorTrait;
use service::sea_orm::Database;
use service::sea_orm::DatabaseConnection;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::async_runtime;
use tauri::image::Image;
use tauri::tray::ClickType;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::State;
use tauri::WindowEvent;
use tauri_plugin_positioner::Position;
use tauri_plugin_positioner::WindowExt;
use transcribe::{get_complete_transcription, get_real_time_transcription};

use crate::device_listener::ActiveListener;
use crate::media::set_configurator_id;
use commands::conversation::{
    create_conversation, delete_conversation, get_conversation, get_conversations,
};
use media::{
    enumerate_audio_input_devices, enumerate_audio_output_devices, set_target_output_device,
};
use recorder::{delete_recording_data, start_recording, stop_recording, RecordingState};
#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = set_configurator_id();

    let device_id = get_default_device_id(true).expect("Failed to get default device");
    let (tx, mut rx) = tokio::sync::watch::channel(false);

    let mut listener_pb = ActiveListener::new(device_id, tx);
    listener_pb.register().expect("Failed to register listener");

    fn handle_ffmpeg_installation() -> FfmpegResult<()> {
        if ffmpeg_is_installed() {
            println!("FFmpeg is already installed! 🎉");
            return Ok(());
        }

        match check_latest_version() {
            Ok(version) => println!("Latest available version: {}", version),
            Err(_) => println!("Skipping version check on this platform."),
        }

        let download_url = ffmpeg_download_url()?;
        let destination = sidecar_dir()?;

        println!("Downloading from: {:?}", download_url);
        let archive_path = download_ffmpeg_package(download_url, &destination)?;
        println!("Downloaded package: {:?}", archive_path);

        println!("Extracting...");
        unpack_ffmpeg(&archive_path, &destination)?;

        let version = ffmpeg_version()?;
        println!("FFmpeg version: {}", version);

        println!("Done! 🏁");
        Ok(())
    }

    handle_ffmpeg_installation().expect("Failed to install FFmpeg");

    let bundle = get_bundle_identifier_or_default("com.devgould.platy");
    set_application(&bundle).unwrap();

    send_notification(
        "Danger",
        Some("Will Robinson"),
        "Run away as fast as you can",
        None,
    )
    .expect("failed to send notification");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(move |app| {
            let tray_window = Arc::new(app.app_handle().get_webview_window("tray-window").unwrap());
            let _ = tray_window.hide();
            let win_clone = tray_window.clone();
            tray_window.on_window_event(move |event| match event {
                WindowEvent::Focused(false) => {
                    let _ = win_clone.hide();
                }
                _ => {}
            });

            let app_window = app.app_handle().get_webview_window("app-window").unwrap();
            let _ = app_window.show();
            TrayIconBuilder::with_id("my-tray")
                .icon(Image::from_path("./icons/icon.png")?)
                .on_tray_icon_event(|app, event| {
                    tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
                    match event.click_type {
                        ClickType::Left => {
                            let tray_window =
                                app.app_handle().get_webview_window("tray-window").unwrap();
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
                .build(app)?;

            let handle = app.handle();

            let data_directory = handle.path().app_data_dir().unwrap();
            let data_directory_clone = data_directory.clone();
            let data_dir_str = data_directory_clone
                .to_str()
                .expect("failed to convert data dir to string");

            let recording_state = Arc::new(tauri::async_runtime::Mutex::new(RecordingState {
                media_process: None,
                recording_options: None,
                shutdown_flag: Arc::new(AtomicBool::new(false)),
                audio_uploading_finished: Arc::new(AtomicBool::new(false)),
                data_dir: Some(data_directory),
            }));
            let recording_state_clone = recording_state.clone();

            app.manage(recording_state);

            println!("Listening for microphone state changes...");

            tauri::async_runtime::spawn(async move {
                loop {
                    if *rx.borrow() {
                        println!("Device is alive");
                    } else {
                        println!("Device is not alive");
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });

            let db_url = "sqlite://".to_string() + data_dir_str + "/db.sqlite?mode=rwc";

            let db = async_runtime::block_on(Database::connect(db_url))
                .expect("Database connection failed");

            async_runtime::block_on(Migrator::up(&db, None)).unwrap();

            let state = AppState { db };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_real_time_transcription,
            get_complete_transcription,
            delete_recording_data,
            enumerate_audio_input_devices,
            enumerate_audio_output_devices,
            set_target_output_device,
            get_conversation,
            get_conversations,
            create_conversation,
            delete_conversation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
