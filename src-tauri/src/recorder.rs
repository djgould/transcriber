use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::State;
use tokio::process::Command;

use crate::media::MediaRecorder;
use crate::transcribe::{start_transcription_loop, transcribe_wav_file};
use crate::utils::ffmpeg_path_as_str;

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
use tokio::io::AsyncBufReadExt;

async fn combine_segments(
    audio_chunks_dir: PathBuf,
) -> Result<tokio::process::Child, std::io::Error> {
    let ffmpeg_binary_path_str = ffmpeg_path_as_str().unwrap().to_owned();

    let segment_list_path = audio_chunks_dir.join("segment_list.txt");

    // Read each line (segment file path) from the segment list file
    let segment_files: Vec<String> = match std::fs::read_to_string(&segment_list_path) {
        Ok(content) => Some(
            content
                .lines()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>(),
        ),
        Err(e) => {
            eprintln!("Failed to read segment list: {}", e);
            None
        }
    }
    .expect("Failed to read segment list. This should never happen. Please report this bug.");

    // Ensure there are segments to combine
    if segment_files.is_empty() {
        eprintln!("No segments found to combine.");
    }

    let concat_file_path = audio_chunks_dir.join("concat.txt").clone();
    let combined_output_file_path = audio_chunks_dir.join("combined.wav");

    write_concat_file(&concat_file_path, &segment_files).expect("error writing concat file");

    let args = vec![
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        concat_file_path.to_str().unwrap(),
        "-c",
        "copy",
        combined_output_file_path.to_str().unwrap(),
    ];

    // Print the generated args for debugging
    println!("FFmpeg args: {:?}", args);

    let mut process = Command::new(ffmpeg_binary_path_str).args(args).spawn()?;

    if let Some(process_stderr) = process.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::BufReader;

            let mut process_reader = BufReader::new(process_stderr).lines();
            while let Ok(Some(line)) = process_reader.next_line().await {
                eprintln!("FFmpeg process STDERR: {}", line);
            }
        });
    }

    process.wait().await?;
    Ok(process)
}

fn write_concat_file(concat_file_path: &PathBuf, segment_files: &Vec<String>) -> io::Result<()> {
    let mut output_file = File::create(concat_file_path)?;
    for segment_file in segment_files {
        output_file
            .write_all(format!("file '{}'\n", segment_file).as_bytes())
            .expect("error writing file");
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

    while !guard.audio_uploading_finished.load(Ordering::SeqCst) {
        println!("Waiting for uploads to finish...");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let data_dir = guard.data_dir.clone();

    combine_segments(data_dir.expect("no data directory").join("chunks/audio"))
        .await
        .map_err(|e| e.to_string())?;

    tokio::time::sleep(Duration::from_millis(50)).await;

    println!("combined segments..");

    let combined_audio_file = guard
        .data_dir
        .clone()
        .expect("no data directory")
        .join("chunks/audio/combined.wav");
    let transcription_output_file = guard
        .data_dir
        .clone()
        .expect("no data directory")
        .join("chunks/audio/transcription.json");
    transcribe_wav_file(&combined_audio_file, &transcription_output_file)
        .map_err(|e| e.to_string())?;

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
