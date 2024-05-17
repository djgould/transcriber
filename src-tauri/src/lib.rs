// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod media;
mod recorder;
mod utils;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::DevicesError;
use cpal::Stream;
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::download::check_latest_version;
use ffmpeg_sidecar::download::download_ffmpeg_package;
use ffmpeg_sidecar::download::ffmpeg_download_url;
use ffmpeg_sidecar::download::unpack_ffmpeg;
use ffmpeg_sidecar::error::Result as FfmpegResult;
use ffmpeg_sidecar::paths::sidecar_dir;
use ffmpeg_sidecar::version::ffmpeg_version;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use serde::Serialize;
use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{Manager, State};
use tauri_plugin_sql::PluginConfig;
use tauri_plugin_sql::{Builder, Migration, MigrationKind};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use recorder::{start_recording, stop_recording, RecordingState, TranscriptionJSON};

fn parse_wav_file(path: &Path) -> Vec<i16> {
    let reader = WavReader::open(path).expect("failed to read file");

    if reader.spec().channels != 1 {
        panic!("expected mono audio file");
    }
    if reader.spec().sample_format != SampleFormat::Int {
        panic!("expected integer sample format");
    }
    if reader.spec().bits_per_sample != 16 {
        panic!("expected 16 bits per sample");
    }

    reader
        .into_samples::<i16>()
        .map(|x| x.expect("sample"))
        .collect::<Vec<_>>()
}

fn parse_and_resample_wav_file(path: &Path, target_sample_rate: f64) -> Vec<i16> {
    let mut reader = WavReader::open(path).expect("failed to read file");
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("expected mono audio file");
    }
    if spec.sample_format != SampleFormat::Int {
        panic!("expected integer sample format");
    }
    if spec.bits_per_sample != 16 {
        panic!("expected 16 bits per sample");
    }

    // Original sample rate
    let original_sample_rate = spec.sample_rate as f64;

    // Read all samples
    let samples: Vec<i16> = reader
        .samples::<i16>()
        .map(|s| s.expect("failed to read sample"))
        .collect();

    // Set up resampler if the sample rates are different
    let resampled_samples = if (spec.sample_rate as f64 - target_sample_rate).abs() > f64::EPSILON {
        resample_audio(samples, spec.sample_rate, target_sample_rate, spec.channels)
    } else {
        samples
    };

    // Save the resampled audio to a new file
    // save_to_wav(
    //     &resampled_samples,
    //     spec.channels,
    //     target_sample_rate as u32,
    //     "output_resampled.wav",
    // );

    resampled_samples
}

fn resample_audio(
    samples: Vec<i16>,
    original_rate: u32,
    target_rate: f64,
    channels: u16,
) -> Vec<i16> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.90,
        interpolation: rubato::SincInterpolationType::Cubic,
        oversampling_factor: 256,
        window: rubato::WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(
        target_rate / original_rate as f64,
        2.0,
        params,
        samples.len(),
        1, // Channels
    )
    .unwrap();

    // Convert i16 to f32 samples
    let f32_samples: Vec<f32> = samples
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect();

    let waves_in = &[f32_samples];
    // Resample
    let resampled_samples = resampler.process(waves_in, None).unwrap();

    // Convert back to i16
    return resampled_samples[0]
        .iter()
        .map(|&s| (s * i16::MAX as f32) as i16)
        .collect();
}

// // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// async fn transcribe(path: String) -> Result<Vec<String>, String> {
//     tokio::task::spawn_blocking(move || {
//         use std::path::Path;

//         println!("Path: {}", path);
//         let audio_path = Path::new("/Users/devingould/platy/src-tauri/src/samples/a13.wav");
//         if !audio_path.exists() {
//             panic!("audio file doesn't exist");
//         }
//         let whisper_path =
//             Path::new("/Users/devingould/platy/src-tauri/src/models/ggml-small.en-tdrz.bin");
//         if !whisper_path.exists() {
//             panic!("whisper file doesn't exist");
//         }

//         // Assuming parse_wav_file and other functions are correctly defined elsewhere
//         let original_samples = parse_and_resample_wav_file(audio_path, 16000.0);
//         let mut samples = vec![0.0f32; original_samples.len()];
//         whisper_rs::convert_integer_to_float_audio(&original_samples, &mut samples)
//             .expect("failed to convert samples");

//         let ctx = WhisperContext::new_with_params(
//             &whisper_path.to_string_lossy(),
//             WhisperContextParameters::default(),
//         )
//         .expect("failed to open model");
//         let mut state = ctx.create_state().expect("failed to create state");
//         let mut params = FullParams::new(SamplingStrategy::default());
//         params.set_initial_prompt("experience");
//         params.set_progress_callback_safe(|progress| println!("Progress callback: {}%", progress));
//         params.set_tdrz_enable(true);

//         let st = std::time::Instant::now();
//         state
//             .full(params, &samples)
//             .expect("failed to transcribe audio");

//         let et = std::time::Instant::now();

//         let num_segments = state
//             .full_n_segments()
//             .expect("failed to get number of segments");
//         let mut full_text: Vec<String> = vec![String::new()];
//         let mut full_text_index = 0;
//         for i in 0..num_segments {
//             let segment = state
//                 .full_get_segment_text(i)
//                 .expect("failed to get segment");
//             full_text[full_text_index].push_str(&segment);
//             if (state.full_get_segment_speaker_turn_next(i)) {
//                 full_text.push(String::new());
//                 full_text_index += 1
//             }
//             let start_timestamp = state
//                 .full_get_segment_t0(i)
//                 .expect("failed to get start timestamp");
//             let end_timestamp = state
//                 .full_get_segment_t1(i)
//                 .expect("failed to get end timestamp");
//             println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
//         }
//         println!("Transcription took {}ms", (et - st).as_millis());
//         Ok(full_text)
//     })
//     .await
//     .map_err(|e| e.to_string())?
// }

// #[tauri::command]
// fn start_recording(audio_controller: tauri::State<'_, Arc<audio_controller::AudioController>>) {
//     audio_controller.start();
// }

// #[tauri::command]
// fn stop_recording(audio_controller: tauri::State<'_, Arc<audio_controller::AudioController>>) {
//     audio_controller.stop();
// }

#[tauri::command]
async fn get_real_time_transcription(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
) -> Result<TranscriptionJSON, String> {
    let mut state_guard = state.lock().await;

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let data_dir = match &state_guard.data_dir {
        Some(dir) => dir,
        None => return Err("Data directory not set".to_string()),
    };

    let audio_dir = data_dir.join("chunks/audio");

    let mut paths: Vec<PathBuf> = match fs::read_dir(audio_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.file_name() != Some(std::ffi::OsStr::new("transcription.json")))
            .collect(),
        Err(err) => return Err(format!("Failed to read directory: {}", err)),
    };

    paths.sort_by(|a, b| {
        let a_name = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    let mut merged_content = TranscriptionJSON {
        full_text: Vec::new(),
    };

    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .map_err(|err| format!("Failed to read file {}: {}", path.display(), err))?;

            let json_content: TranscriptionJSON =
                serde_json::from_str(&content).map_err(|err| {
                    format!("Failed to parse JSON in file {}: {}", path.display(), err)
                })?;

            merged_content.full_text.extend(json_content.full_text);
        }
    }

    Ok(merged_content)
}

#[tauri::command]
async fn get_complete_transcription(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
) -> Result<TranscriptionJSON, String> {
    let mut state_guard = state.lock().await;

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let data_dir = match &state_guard.data_dir {
        Some(dir) => dir,
        None => return Err("Data directory not set".to_string()),
    };

    let audio_dir = data_dir.join("chunks/audio");

    let mut paths: Vec<PathBuf> = match fs::read_dir(audio_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.file_name() == Some(std::ffi::OsStr::new("transcription.json")))
            .collect(),
        Err(err) => return Err(format!("Failed to read directory: {}", err)),
    };

    println!("Found {} transcription files", paths.len());

    paths.sort_by(|a, b| {
        let a_name = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    let mut merged_content = TranscriptionJSON {
        full_text: Vec::new(),
    };

    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .map_err(|err| format!("Failed to read file {}: {}", path.display(), err))?;

            let json_content: TranscriptionJSON =
                serde_json::from_str(&content).map_err(|err| {
                    format!("Failed to parse JSON in file {}: {}", path.display(), err)
                })?;

            merged_content.full_text.extend(json_content.full_text);
        }
    }

    Ok(merged_content)
}

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
    ];

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

    // let transcriber_controller = Arc::new(transcribe::TranscriberController::new());
    // let audio_controller = Arc::new(audio_controller::AudioController::new(
    //     &transcriber_controller,
    // ));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:test.db", migrations)
                .build(),
        )
        .setup(move |app| {
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
        // .manage(audio_controller)
        // .manage(transcriber_controller)
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_real_time_transcription,
            get_complete_transcription,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
