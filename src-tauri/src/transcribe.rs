use std::{
    collections::HashSet,
    fs::{read_dir, read_to_string, File},
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::future::join_all;
use hound::{SampleFormat, WavReader};
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::{recorder::RecordingState, utils::load_segment_list};

#[derive(Serialize, Deserialize)]
pub struct TranscriptionJSON {
    pub full_text: Vec<String>,
}

pub fn transcribe_wav_file(
    wav_filepath: &PathBuf,
    transcription_output_file_path: &PathBuf,
) -> Result<(), String> {
    let filepath_str = wav_filepath.to_str().unwrap_or_default().to_owned();
    println!("{}", filepath_str);
    use std::path::Path;

    let whisper_path =
        Path::new("/Users/devingould/platy/src-tauri/src/models/ggml-small.en-tdrz.bin");
    if !whisper_path.exists() {
        panic!("whisper file doesn't exist");
    }

    let mut reader = WavReader::open(filepath_str).expect("failed to read file");
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
    params.set_progress_callback_safe(|progress| println!("Progress callback: {}%", progress));
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
        if state.full_get_segment_speaker_turn_next(i) {
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

    let json_string =
        serde_json::to_string_pretty(&transcription).expect("failed to serialize transcription");

    let mut file = File::create(transcription_output_file_path).expect("couldn't create file");
    file.write_all(json_string.as_bytes())
        .expect("could not write to file");
    Ok(())
}

pub async fn start_transcription_loop(
    chunks_dir: PathBuf,
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
                let segment_path_clone = segment_path.clone();
                transcription_tasks.push(tokio::spawn(async move {
                    transcribe_wav_file(&segment_path_clone, &transcription_path)?;

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

#[tauri::command]
pub async fn get_real_time_transcription(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
) -> Result<TranscriptionJSON, String> {
    let state_guard = state.lock().await;

    let data_dir = match &state_guard.data_dir {
        Some(dir) => dir,
        None => return Err("Data directory not set".to_string()),
    };

    let audio_dir = data_dir.join("chunks/audio");

    let mut paths: Vec<PathBuf> = match read_dir(audio_dir) {
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
            let content = read_to_string(&path)
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
pub async fn get_complete_transcription(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
    conversation_id: u64,
) -> Result<TranscriptionJSON, String> {
    let state_guard = state.lock().await;

    let data_dir = match &state_guard.data_dir {
        Some(dir) => dir,
        None => return Err("Data directory not set".to_string()),
    };

    let audio_dir = data_dir
        .join("chunks/audio")
        .join(conversation_id.to_string());

    let mut paths: Vec<PathBuf> = match read_dir(audio_dir) {
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
            let content = read_to_string(&path)
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
