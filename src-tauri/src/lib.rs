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
    create_input_aggregate_device, create_output_aggregate_device,
};
use audio::macos::helpers::{check_device_exists, get_device_uid};
use coreaudio::audio_unit::macos_helpers::{
    get_default_device_id, get_device_id_from_name, get_device_name,
};
use coreaudio_sys::{AudioDeviceID, AudioObjectID};
use log::{error, info};
use migration::Migrator;
use migration::MigratorTrait;
use service::mutation::Mutation;
use service::sea_orm::Database;
use service::sea_orm::DatabaseConnection;
use service::sea_orm::TryIntoModel;
use tauri::async_runtime;
use tauri::image::Image;
use tauri::tray::TrayIconBuilder;
use tauri::tray::TrayIconEvent;
use tauri::Manager;
use tauri::WindowEvent;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_positioner::Position;
use tauri_plugin_positioner::WindowExt;
use transcribe::{get_complete_transcription, get_real_time_transcription};
use uuid::Uuid;

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

use std::sync::{atomic::AtomicBool, Arc};

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

struct DeviceState {
    active_listener: ActiveListener,
    selected_input_name: Option<String>,
    selected_output_name: Option<String>,
    input_device_id: Option<AudioDeviceID>,
    aggregate_device_id: Option<AudioDeviceID>,
    tap_id: Option<AudioObjectID>,
    output_device_id: Option<AudioDeviceID>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, mut rx) = tokio::sync::watch::channel(false);

    std::panic::set_hook(Box::new(|info| {
        eprintln!("Panicked: {:?}", info);
        error!("Panicked: {:?}", info);
    }));

    let device_id = get_default_device_id(true).expect("Failed to get default device");
    let default_output_device_id =
        get_default_device_id(false).expect("Failed to get the default output device");
    let default_input_name =
        get_device_name(device_id).expect("Failed to get the default device name");
    let default_output_name =
        get_device_name(default_output_device_id).expect("Failed to get the default device name");

    let device_id = get_default_device_id(false).expect("failed to get default device");
    let device_uid = get_device_uid(device_id).expect("failed to get device uid");
    let aggregate_device_result =
        create_output_aggregate_device(&device_uid, "Platy Speaker", &Uuid::new_v4().to_string())
            .expect("failed to create aggregate device");

    let device_exists = check_device_exists("Platy Microphone");
    if !device_exists {
        info!("Aggregate microphone device not found, creating one");
        create_input_aggregate_device("BuiltInMicrophoneDevice")
            .expect("failed to create aggregate device");
    } else {
        info!("Aggregate microphone already exists");
    }

    let input_device_id =
        get_device_id_from_name("Platy Microphone", true).expect("Platy Microphone doesn't exist");

    let mut listener = ActiveListener::new(tx);
    listener
        .register(input_device_id)
        .expect("Failed to register listener");

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
            let tray_window = Arc::new(app.app_handle().get_webview_window("tray-window").unwrap());
            let _ = tray_window.set_visible_on_all_workspaces(true);
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
                .icon(Image::from_path(
                    app.app_handle()
                        .path()
                        .resource_dir()
                        .expect("failed to get resource dir")
                        .join("icons/icon.ico"),
                )?)
                .on_tray_icon_event(|app, event| {
                    tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
                    match event {
                        TrayIconEvent::Click { .. } => {
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
                tap_id: Some(aggregate_device_result.tap_id),
                input_device_id: Some(input_device_id),
                aggregate_device_id: Some(aggregate_device_result.aggregate_device_id),
                output_device_id: Some(device_id),
            };
            app.manage(Arc::new(tauri::async_runtime::Mutex::new(device_state)));

            let _app_handle = app.handle().clone();

            info!("Listening for microphone state changes...");
            tauri::async_runtime::spawn(async move {
                while rx.changed().await.is_ok() {
                    let device_alive = *rx.borrow();
                    if device_alive {
                        let app_state: tauri::State<AppState> = _app_handle.state();
                        let recording_state: tauri::State<
                            Arc<tauri::async_runtime::Mutex<RecordingState>>,
                        > = _app_handle.state();
                        let device_state: tauri::State<
                            Arc<tauri::async_runtime::Mutex<DeviceState>>,
                        > = _app_handle.state();

                        let should_start_recording = {
                            let recording_guard = recording_state.lock().await;
                            recording_guard.media_process.is_none()
                        };

                        if should_start_recording {
                            info!("Device is alive, starting recording");
                            let conversation = Mutation::create_conversation(
                                &app_state.db,
                                entity::conversation::Model {
                                    title: "New Conversation".to_string(),
                                    id: 0,
                                    created_at: String::new(),
                                    updated_at: String::new(),
                                },
                            )
                            .await
                            .expect("Failed to insert conversation")
                            .try_into_model()
                            .expect("Failed to convert active model into model");

                            _start_recording(
                                recording_state.clone(),
                                device_state.clone(),
                                RecordingOptions {
                                    user_id: "user".to_string(),
                                    audio_input_name: "default".to_string(),
                                    audio_output_name: "default".to_string(),
                                },
                                conversation.id.try_into().unwrap(),
                            )
                            .await
                            .expect("Failed to start recording");
                        } else {
                            info!("Device is alive, recording already running");
                        }
                    } else {
                        let recording_state: tauri::State<
                            Arc<tauri::async_runtime::Mutex<RecordingState>>,
                        > = _app_handle.state();

                        let should_stop_recording = {
                            let recording_guard = recording_state.lock().await;
                            recording_guard.media_process.is_some()
                        };

                        if should_stop_recording {
                            info!("Device is not alive, stopping recording");
                            _stop_recording(_app_handle.clone(), recording_state.clone())
                                .await
                                .expect("Failed to stop recording");
                        } else {
                            info!("Device is not alive, no recording running");
                        }
                    }
                }
                info!("Device listener has been dropped, exiting");
            });

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
