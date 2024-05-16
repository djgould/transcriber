use ffmpeg_sidecar::error::Error;
use futures::future::join_all;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::State;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::media::MediaRecorder;

pub struct RecordingState {
    pub media_process: Option<MediaRecorder>,
    pub recording_options: Option<RecordingOptions>,
    pub shutdown_flag: Arc<AtomicBool>,
    pub audio_uploading_finished: Arc<AtomicBool>,
    pub data_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingOptions {
    pub user_id: String,
    pub audio_name: String,
}

#[tauri::command]
pub async fn start_recording(
    state: State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
    options: RecordingOptions,
) -> Result<(), String> {
    println!("Starting screen recording...");
    let mut state_guard = state.lock().await;

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let data_dir = state_guard
        .data_dir
        .as_ref()
        .ok_or("Data directory is not set in the recording state".to_string())?
        .clone();

    println!("data_dir: {:?}", data_dir);

    let audio_chunks_dir = data_dir.join("chunks/audio");

    clean_and_create_dir(&audio_chunks_dir)?;

    let audio_name = if options.audio_name.is_empty() {
        None
    } else {
        Some(options.audio_name.clone())
    };

    let media_recording_preparation =
        prepare_media_recording(&options, &audio_chunks_dir, audio_name);
    let media_recording_result = media_recording_preparation
        .await
        .map_err(|e| e.to_string())?;

    state_guard.media_process = Some(media_recording_result);
    state_guard.recording_options = Some(options.clone());
    state_guard.shutdown_flag = shutdown_flag.clone();
    state_guard.audio_uploading_finished = Arc::new(AtomicBool::new(false));

    let audio_upload = start_transcription_loop(
        audio_chunks_dir,
        options.clone(),
        shutdown_flag.clone(),
        state_guard.audio_uploading_finished.clone(),
    );

    drop(state_guard);

    println!("Starting upload loops...");

    match tokio::try_join!(audio_upload) {
        Ok(_) => {
            println!("Both upload loops completed successfully.");
        }
        Err(e) => {
            eprintln!("An error occurred: {}", e);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, Arc<Mutex<RecordingState>>>) -> Result<(), String> {
    let mut guard = state.lock().await;

    println!("Stopping media recording...");

    guard.shutdown_flag.store(true, Ordering::SeqCst);

    if let Some(mut media_process) = guard.media_process.take() {
        println!("Stopping media recording...");
        media_process
            .stop_media_recording()
            .await
            .expect("Failed to stop media recording");
    }

    // let is_local_mode = match dotenv_codegen::dotenv!("NEXT_PUBLIC_LOCAL_MODE") {
    //     "true" => true,
    //     _ => false,
    // };

    // if !is_local_mode {
    //     while !guard.audio_uploading_finished.load(Ordering::SeqCst) {
    //         println!("Waiting for uploads to finish...");
    //         tokio::time::sleep(Duration::from_millis(50)).await;
    //     }
    // }

    println!("All recordings and uploads stopped.");

    Ok(())
}

fn clean_and_create_dir(dir: &Path) -> Result<(), String> {
    if dir.exists() {
        // Instead of just reading the directory, this will also handle subdirectories.
        std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    if !dir.to_string_lossy().contains("screenshots") {
        let segment_list_path = dir.join("segment_list.txt");
        match File::open(&segment_list_path) {
            Ok(_) => Ok(()),
            Err(ref e) if e.kind() == ErrorKind::NotFound => {
                File::create(&segment_list_path).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TranscriptionJSON {
    pub full_text: Vec<String>,
}

async fn start_transcription_loop(
    chunks_dir: PathBuf,
    options: RecordingOptions,
    shutdown_flag: Arc<AtomicBool>,
    transcription_finished: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut watched_segments: HashSet<String> = HashSet::new();
    let mut is_final_loop = false;

    loop {
        let mut transcription_tasks: Vec<tokio::task::JoinHandle<Result<(), String>>> = vec![];
        if shutdown_flag.load(Ordering::SeqCst) {
            if is_final_loop {
                break;
            }
            is_final_loop = true;
        }

        let current_segments = load_segment_list(&chunks_dir.join("segment_list.txt"))
            .map_err(|e| e.to_string())?
            .difference(&watched_segments)
            .cloned()
            .collect::<HashSet<String>>();

        for segment_filename in &current_segments {
            let segment_path = chunks_dir.join(segment_filename);
            let transcription_path = chunks_dir.join(format!("{}.json", segment_filename));
            if segment_path.is_file() {
                let options_clone = options.clone();
                let segment_path_clone = segment_path.clone();
                transcription_tasks.push(tokio::spawn(async move {
                    let filepath_str = segment_path_clone.to_str().unwrap_or_default().to_owned();
                    use std::path::Path;

                    let whisper_path = Path::new(
                        "/Users/devingould/platy/src-tauri/src/models/ggml-small.en-tdrz.bin",
                    );
                    if !whisper_path.exists() {
                        panic!("whisper file doesn't exist");
                    }

                    let mut reader =
                        WavReader::open(segment_path_clone).expect("failed to read file");
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
                    let original_samples: Vec<i16> = reader
                        .samples::<i16>()
                        .map(|s| s.expect("failed to read sample"))
                        .collect();
                    let mut samples = vec![0.0f32; original_samples.len()];

                    whisper_rs::convert_integer_to_float_audio(&original_samples, &mut samples)
                        .expect("failed to convert samples");

                    let ctx = WhisperContext::new_with_params(
                        &whisper_path.to_string_lossy(),
                        WhisperContextParameters::default(),
                    )
                    .expect("failed to open model");
                    let mut state = ctx.create_state().expect("failed to create state");
                    let mut params = FullParams::new(SamplingStrategy::default());
                    params.set_initial_prompt("experience");
                    params.set_progress_callback_safe(|progress| {
                        println!("Progress callback: {}%", progress)
                    });
                    params.set_tdrz_enable(true);

                    let st = std::time::Instant::now();
                    state
                        .full(params, &samples)
                        .expect("failed to transcribe audio");

                    let et = std::time::Instant::now();

                    let num_segments = state
                        .full_n_segments()
                        .expect("failed to get number of segments");
                    let mut full_text: Vec<String> = vec![String::new()];
                    let mut full_text_index = 0;
                    for i in 0..num_segments {
                        let segment = state
                            .full_get_segment_text(i)
                            .expect("failed to get segment");
                        full_text[full_text_index].push_str(&segment);
                        if (state.full_get_segment_speaker_turn_next(i)) {
                            full_text.push(String::new());
                            full_text_index += 1
                        }
                        let start_timestamp = state
                            .full_get_segment_t0(i)
                            .expect("failed to get start timestamp");
                        let end_timestamp = state
                            .full_get_segment_t1(i)
                            .expect("failed to get end timestamp");
                        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
                    }
                    println!("Transcription took {}ms", (et - st).as_millis());

                    let transcription = TranscriptionJSON {
                        full_text: full_text,
                    };

                    let json_string = serde_json::to_string_pretty(&transcription)
                        .expect("failed to serialize transcription");

                    let mut file = File::create(transcription_path).expect("couldn't create file");
                    file.write_all(json_string.as_bytes());
                    Ok(())
                }));
            }
            watched_segments.insert(segment_filename.clone());
        }

        if !transcription_tasks.is_empty() {
            let _ = join_all(transcription_tasks).await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    transcription_finished.store(true, Ordering::SeqCst);
    Ok(())
}

fn load_segment_list(segment_list_path: &Path) -> io::Result<HashSet<String>> {
    let file = File::open(segment_list_path)?;
    let reader = BufReader::new(file);

    let mut segments = HashSet::new();
    for line_result in reader.lines() {
        let line = line_result?;
        if !line.is_empty() {
            segments.insert(line);
        }
    }

    Ok(segments)
}

async fn prepare_media_recording(
    options: &RecordingOptions,
    audio_chunks_dir: &Path,
    audio_name: Option<String>,
) -> Result<MediaRecorder, String> {
    let mut media_recorder = MediaRecorder::new();
    let audio_file_path = audio_chunks_dir.to_str().unwrap();
    media_recorder
        .start_media_recording(
            options.clone(),
            audio_file_path,
            audio_name.as_ref().map(String::as_str),
        )
        .await?;
    Ok(media_recorder)
}
