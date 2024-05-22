// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod media;
mod recorder;
mod transcribe;
mod utils;

use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::download::check_latest_version;
use ffmpeg_sidecar::download::download_ffmpeg_package;
use ffmpeg_sidecar::download::ffmpeg_download_url;
use ffmpeg_sidecar::download::unpack_ffmpeg;
use ffmpeg_sidecar::error::Result as FfmpegResult;
use ffmpeg_sidecar::paths::sidecar_dir;
use ffmpeg_sidecar::version::ffmpeg_version;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::image::Image;
use tauri::tray::ClickType;
use tauri::tray::TrayIconBuilder;
use tauri::ActivationPolicy;
use tauri::Manager;
use tauri::WindowEvent;
use tauri_plugin_positioner::Position;
use tauri_plugin_positioner::WindowExt;
use tauri_plugin_sql::{Migration, MigrationKind};
use transcribe::{get_complete_transcription, get_real_time_transcription};

use media::{
    enumerate_audio_input_devices, enumerate_audio_output_devices, set_target_output_device,
};
use recorder::{delete_recording_data, start_recording, stop_recording, RecordingState};

use crate::media::set_configurator_id;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_metting_table",
            sql: "CREATE TABLE meetings (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                transcription TEXT NOT NULL, -- Store JSON data as text
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_conversations_table",
            sql: "CREATE TABLE conversations (
                id INTEGER PRIMARY KEY,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            kind: MigrationKind::Up,
        },
    ];

    set_configurator_id();

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

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:test.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_positioner::init())
        .setup(move |app| {
            let tray_window = Arc::new(app.app_handle().get_webview_window("tray-window").unwrap());
            tray_window.hide();
            let win_clone = tray_window.clone();
            tray_window.on_window_event(move |event| match event {
                WindowEvent::Focused(false) => {
                    win_clone.hide();
                }
                _ => {}
            });

            let app_window = app.app_handle().get_webview_window("app-window").unwrap();
            app_window.show();
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

            let recording_state = RecordingState {
                media_process: None,
                recording_options: None,
                shutdown_flag: Arc::new(AtomicBool::new(false)),
                audio_uploading_finished: Arc::new(AtomicBool::new(false)),
                data_dir: Some(data_directory),
            };

            app.manage(Arc::new(tauri::async_runtime::Mutex::new(recording_state)));

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
            set_target_output_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
