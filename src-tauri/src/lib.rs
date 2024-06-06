// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod commands;
mod device_listener;
mod media;
mod recorder;
mod summarize;
mod transcribe;
mod utils;

use audio::macos::aggregate_device::{
    self, create_input_aggregate_device, create_output_aggregate_device,
};
use audio::macos::helpers::{all_device_uids, check_device_exists, get_device_uid};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use coreaudio::audio_unit::macos_helpers::{
    get_default_device_id, get_device_id_from_name, get_device_name,
};
use coreaudio_sys::AudioDeviceID;
use entity::conversation::Model;
use entity::conversation::{self, Model as ConversationModel};
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::download::check_latest_version;
use ffmpeg_sidecar::download::download_ffmpeg_package;
use ffmpeg_sidecar::download::ffmpeg_download_url;
use ffmpeg_sidecar::download::unpack_ffmpeg;
use ffmpeg_sidecar::error::Result as FfmpegResult;
use ffmpeg_sidecar::paths::sidecar_dir;
use ffmpeg_sidecar::version::ffmpeg_version;
use log::{error, info};
use mac_notification_sys::get_bundle_identifier_or_default;
use mac_notification_sys::set_application;
use migration::Migrator;
use migration::MigratorTrait;
use service::sea_orm::{Database, TryIntoModel};
use service::sea_orm::{DatabaseConnection, Set};
use service::Mutation;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::async_runtime;
use tauri::image::Image;
use tauri::tray::ClickType;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::WindowEvent;
use tauri_plugin_log::{Target, TargetKind};
// use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_positioner::Position;
use tauri_plugin_positioner::WindowExt;
use transcribe::{get_complete_transcription, get_real_time_transcription};

use crate::device_listener::ActiveListener;
use crate::recorder::{RecordingOptions, _start_recording, _stop_recording};
use commands::{
    conversation::{
        create_conversation, delete_conversation, get_conversation, get_conversations,
        get_summary_for_converstation, open_conversation,
    },
    devices::{
        enumerate_audio_input_devices, enumerate_audio_output_devices, set_input_device_name,
        set_output_device_name,
    },
    recording::is_recording,
};
use media::set_target_output_device;
use recorder::{delete_recording_data, start_recording, stop_recording, RecordingState};
#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

struct DeviceState {
    active_listener: ActiveListener,
    selected_input_name: Option<String>,
    selected_output_name: Option<String>,
    aggregate_device_id: Option<AudioDeviceID>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, mut rx) = tokio::sync::watch::channel(false);

    std::panic::set_hook(Box::new(|info| {
        eprintln!("Panicked: {:?}", info);
        error!("Panicked: {:?}", info);
    }));

    ffmpeg_sidecar::download::auto_download().expect("Failed to download ffmpeg");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .setup(move |app| {
            let device_id = get_default_device_id(true).expect("Failed to get default device");
            let default_output_device_id =
                get_default_device_id(false).expect("Failed to get the default output device");
            let default_input_name =
                get_device_name(device_id).expect("Failed to get the default device name");
            let default_output_name = get_device_name(default_output_device_id)
                .expect("Failed to get the default device name");

            let device_id = get_default_device_id(false).expect("failed to get default device");
            let device_uid = get_device_uid(device_id).expect("failed to get device uid");
            let aggregate_device_result =
                create_output_aggregate_device(&device_uid, "Platy Speaker", "platy-speaker-1")
                    .expect("failed to create aggregate device");

            let device_exists = check_device_exists("Platy Microphone");
            if !device_exists {
                info!("Aggregate microphone device not found, creating one");
                create_input_aggregate_device("BuiltInMicrophoneDevice")
                    .expect("failed to create aggregate device");
            } else {
                info!("Aggregate microphone already exists");
            }
            let input_device_id = get_device_id_from_name("Platy Microphone", true)
                .expect("Platy Microphone doesn't exist");

            let mut listener = ActiveListener::new(tx);
            listener
                .register(input_device_id)
                .expect("Failed to register listener");

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
                // .icon(Image::from_path("./icons/icon.ico")?)
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

            if !data_directory.exists() {
                info!("data dir doesn't exist");
            }

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
                conversation_id: None,
            }));

            app.manage(recording_state);

            let db_url = "sqlite://".to_string() + data_dir_str + "/db.sqlite?mode=rwc";
            let db = async_runtime::block_on(Database::connect(db_url))
                .expect("Database connection failed");

            async_runtime::block_on(Migrator::up(&db, None)).unwrap();

            let state = AppState { db };
            app.manage(state);

            let device_state = DeviceState {
                selected_input_name: Some(default_input_name),
                selected_output_name: Some(default_output_name),
                active_listener: listener,
                aggregate_device_id: Some(aggregate_device_result.aggregate_device_id),
            };
            app.manage(Arc::new(tauri::async_runtime::Mutex::new(device_state)));

            let _app_handle = app.handle().clone();

            info!("Listening for microphone state changes...");
            // tauri::async_runtime::spawn(async move {
            //     loop {
            //         match async {
            //             if *rx.borrow() {
            //                 let app_state: tauri::State<AppState> = _app_handle.state();
            //                 let recording_state: tauri::State<
            //                     Arc<tauri::async_runtime::Mutex<RecordingState>>,
            //                 > = _app_handle.state();
            //                 let recording_state_clone = recording_state.clone();
            //                 let should_start_recording = {
            //                     let recording_guard = recording_state.lock().await;
            //                     if recording_guard.media_process.is_some() {
            //                         false
            //                     } else {
            //                         true
            //                     }
            //                 };

            //                 let _ = &app_state.db;

            //                 if should_start_recording {
            //                     info!("Device is alive, starting recording");
            //                     let conversation = Mutation::create_conversation(
            //                         &app_state.db,
            //                         entity::conversation::Model {
            //                             title: "bla".to_string(),
            //                             id: 0,
            //                             created_at: String::new(),
            //                             updated_at: String::new(),
            //                         },
            //                     )
            //                     .await
            //                     .expect("could not insert conversation")
            //                     .try_into_model()
            //                     .expect("could not turn active model into model");
            //                     let _ = _start_recording(
            //                         recording_state_clone,
            //                         RecordingOptions {
            //                             user_id: "devin".to_string(),
            //                             audio_input_name: "default".to_string(),
            //                             audio_output_name: "default".to_string(),
            //                         },
            //                         conversation.id.try_into().unwrap(),
            //                     )
            //                     .await;
            //                 } else {
            //                     info!("Device is alive, recording running");
            //                 };
            //             } else {
            //                 let app_state: tauri::State<AppState> = _app_handle.state();
            //                 let recording_state: tauri::State<
            //                     Arc<tauri::async_runtime::Mutex<RecordingState>>,
            //                 > = _app_handle.state();
            //                 let should_stop_recording = {
            //                     let recording_guard = recording_state.lock().await;
            //                     if recording_guard.media_process.is_some() {
            //                         true
            //                     } else {
            //                         false
            //                     }
            //                 };

            //                 let _ = &app_state.db;

            //                 if should_stop_recording {
            //                     info!("Device is not alive, stopping recording");
            //                     // let _ = _stop_recording(recording_state_clone).await;
            //                 } else {
            //                     info!("Device is not alive, no recording running");
            //                 };
            //             }
            //             Ok::<(), ()>(())
            //         }
            //         .await
            //         {
            //             Ok(_) => {}
            //             Err(_) => {
            //                 // Handle the error if necessary, or just log that something went wrong
            //                 info!("An error occurred in the loop iteration, but continuing...");
            //             }
            //         }
            //         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            //     }
            // });

            info!("SETUP SUCCESS");
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
            delete_conversation,
            set_input_device_name,
            set_output_device_name,
            get_summary_for_converstation,
            open_conversation,
            is_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
