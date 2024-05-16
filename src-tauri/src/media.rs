use byteorder::{ByteOrder, LittleEndian};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::io::{Error, ErrorKind::WouldBlock};
use std::path::Path;
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use tauri::async_runtime::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc;

use crate::recorder::RecordingOptions;
use crate::utils::ffmpeg_path_as_str;

const FRAME_RATE: u64 = 30;

unsafe impl Send for MediaRecorder {}
unsafe impl Sync for MediaRecorder {}

pub struct MediaRecorder {
    pub options: Option<RecordingOptions>,
    ffmpeg_audio_process: Option<tokio::process::Child>,
    ffmpeg_audio_stdin: Option<Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
    device_name: Option<String>,
    stream: Option<cpal::Stream>,
    audio_channel_sender: Option<mpsc::Sender<Vec<u8>>>,
    audio_channel_receiver: Option<mpsc::Receiver<Vec<u8>>>,
    should_stop: Arc<AtomicBool>,
    start_time: Option<Instant>,
    audio_file_path: Option<String>,
}

impl MediaRecorder {
    pub fn new() -> Self {
        MediaRecorder {
            options: None,
            ffmpeg_audio_process: None,
            ffmpeg_audio_stdin: None,
            device_name: None,
            stream: None,
            audio_channel_sender: None,
            audio_channel_receiver: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            start_time: None,
            audio_file_path: None,
        }
    }

    pub async fn start_media_recording(
        &mut self,
        options: RecordingOptions,
        audio_file_path: &str,
        custom_device: Option<&str>,
    ) -> Result<(), String> {
        self.options = Some(options.clone());

        println!("Custom device: {:?}", custom_device);

        let host = cpal::default_host();
        let devices = host.devices().expect("Failed to get devices");

        let (audio_tx, audio_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2048);

        let audio_start_time = Arc::new(Mutex::new(None));

        self.audio_channel_sender = Some(audio_tx);
        self.audio_channel_receiver = Some(audio_rx);
        self.ffmpeg_audio_stdin = Some(Arc::new(Mutex::new(None)));

        let audio_channel_sender = self.audio_channel_sender.clone();

        let audio_channel_receiver = Arc::new(Mutex::new(self.audio_channel_receiver.take()));

        let should_stop = Arc::clone(&self.should_stop);

        let mut input_devices = devices.filter_map(|device| {
            let supported_input_configs = device.supported_input_configs();
            if supported_input_configs.is_ok() && supported_input_configs.unwrap().count() > 0 {
                Some(device)
            } else {
                None
            }
        });

        let device = if let Some(custom_device_name) = custom_device {
            input_devices
                .find(|d| {
                    d.name()
                        .map(|name| name == custom_device_name)
                        .unwrap_or(false)
                })
                .unwrap_or_else(|| {
                    host.default_input_device()
                        .expect("No default input device available")
                })
        } else {
            host.default_input_device()
                .expect("No default input device available")
        };

        println!(
            "Using audio device: {}",
            device.name().expect("Failed to get device name")
        );

        let config = device
            .supported_input_configs()
            .expect("Failed to get supported input configs")
            .find(|c| {
                c.sample_format() == SampleFormat::F32
                    || c.sample_format() == SampleFormat::I16
                    || c.sample_format() == SampleFormat::I8
                    || c.sample_format() == SampleFormat::I32
            })
            .unwrap_or_else(|| {
                device
                    .supported_input_configs()
                    .expect("Failed to get supported input configs")
                    .next()
                    .expect("No supported input config")
            })
            .with_max_sample_rate();

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let sample_format = match config.sample_format() {
            SampleFormat::I8 => "s8",
            SampleFormat::I16 => "s16le",
            SampleFormat::I32 => "s32le",
            SampleFormat::F32 => "f32le",
            _ => panic!("Unsupported sample format."),
        };

        println!("Sample rate: {}", sample_rate);
        println!("Channels: {}", channels);
        println!("Sample format: {}", sample_format);

        let ffmpeg_binary_path_str = ffmpeg_path_as_str().unwrap().to_owned();

        println!("FFmpeg binary path: {}", ffmpeg_binary_path_str);

        let audio_file_path_owned = audio_file_path.to_owned();
        let sample_rate_str = sample_rate.to_string();
        let channels_str = channels.to_string();

        let ffmpeg_audio_stdin = self.ffmpeg_audio_stdin.clone();

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        if custom_device != Some("None") {
            println!("Building input stream...");

            let stream_result: Result<cpal::Stream, cpal::BuildStreamError> =
                match config.sample_format() {
                    SampleFormat::I8 => device.build_input_stream(
                        &config.into(),
                        {
                            let audio_start_time = Arc::clone(&audio_start_time);
                            move |data: &[i8], _: &_| {
                                let mut first_frame_time_guard = audio_start_time.try_lock();

                                let bytes =
                                    data.iter().map(|&sample| sample as u8).collect::<Vec<u8>>();
                                if let Some(sender) = &audio_channel_sender {
                                    if sender.try_send(bytes).is_err() {
                                        eprintln!("Channel send error. Dropping data.");
                                    }
                                }

                                if let Ok(ref mut start_time_option) = first_frame_time_guard {
                                    if start_time_option.is_none() {
                                        **start_time_option = Some(Instant::now());

                                        println!("Audio start time captured");
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::I16 => device.build_input_stream(
                        &config.into(),
                        {
                            let audio_start_time = Arc::clone(&audio_start_time);
                            move |data: &[i16], _: &_| {
                                let mut first_frame_time_guard = audio_start_time.try_lock();

                                let mut bytes = vec![0; data.len() * 2];
                                LittleEndian::write_i16_into(data, &mut bytes);
                                if let Some(sender) = &audio_channel_sender {
                                    if sender.try_send(bytes).is_err() {
                                        eprintln!("Channel send error. Dropping data.");
                                    }
                                }

                                if let Ok(ref mut start_time_option) = first_frame_time_guard {
                                    if start_time_option.is_none() {
                                        **start_time_option = Some(Instant::now());

                                        println!("Audio start time captured");
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::I32 => device.build_input_stream(
                        &config.into(),
                        {
                            let audio_start_time = Arc::clone(&audio_start_time);
                            move |data: &[i32], _: &_| {
                                let mut first_frame_time_guard = audio_start_time.try_lock();

                                let mut bytes = vec![0; data.len() * 2];
                                LittleEndian::write_i32_into(data, &mut bytes);
                                if let Some(sender) = &audio_channel_sender {
                                    if sender.try_send(bytes).is_err() {
                                        eprintln!("Channel send error. Dropping data.");
                                    }
                                }

                                if let Ok(ref mut start_time_option) = first_frame_time_guard {
                                    if start_time_option.is_none() {
                                        **start_time_option = Some(Instant::now());

                                        println!("Audio start time captured");
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::F32 => device.build_input_stream(
                        &config.into(),
                        {
                            let audio_start_time = Arc::clone(&audio_start_time);
                            move |data: &[f32], _: &_| {
                                let mut first_frame_time_guard = audio_start_time.try_lock();

                                let mut bytes = vec![0; data.len() * 4];
                                LittleEndian::write_f32_into(data, &mut bytes);
                                if let Some(sender) = &audio_channel_sender {
                                    if sender.try_send(bytes).is_err() {
                                        eprintln!("Channel send error. Dropping data.");
                                    }
                                }

                                if let Ok(ref mut start_time_option) = first_frame_time_guard {
                                    if start_time_option.is_none() {
                                        **start_time_option = Some(Instant::now());

                                        println!("Audio start time captured");
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ),
                    _sample_format => Err(cpal::BuildStreamError::DeviceNotAvailable),
                };

            let stream = stream_result.map_err(|_| "Failed to build input stream")?;
            self.stream = Some(stream);
            self.trigger_play()?;
        }

        println!("Starting audio recording and processing...");
        let audio_output_chunk_pattern =
            format!("{}/audio_recording_%03d.wav", audio_file_path_owned);
        let audio_segment_list_filename = format!("{}/segment_list.txt", audio_file_path_owned);

        let mut audio_filters = Vec::new();

        if channels > 2 {
            audio_filters.push("pan=stereo|FL=FL+0.5*FC|FR=FR+0.5*FC");
        }

        audio_filters.push("loudnorm");

        let mut ffmpeg_audio_command: Vec<String> = vec![
            "-f",
            sample_format,
            "-ar",
            &sample_rate_str,
            "-ac",
            &channels_str,
            "-thread_queue_size",
            "4096",
            "-i",
            "pipe:0",
            "-af",
            "aresample=async=1:min_hard_comp=0.100000:first_pts=0:osr=16000",
            "-c:a",
            "pcm_s16le",
            "-async",
            "1",
            "-f",
            "segment",
            "-segment_time",
            "3",
            "-segment_time_delta",
            "0.01",
            "-segment_list",
            &audio_segment_list_filename,
            "-reset_timestamps",
            "1",
            &audio_output_chunk_pattern,
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        println!("FFmpeg audio command: {:?}", ffmpeg_audio_command.join(" "));

        println!("Starting FFmpeg audio process...");

        let mut audio_stdin: Option<ChildStdin> = None;
        let mut audio_child: Option<Child> = None;

        if custom_device != Some("None") {
            let (child, stdin) = self
                .start_audio_ffmpeg_processes(&ffmpeg_binary_path_str, &ffmpeg_audio_command)
                .await
                .map_err(|e| e.to_string())?;
            audio_child = Some(child);
            audio_stdin = Some(stdin);
            println!("Audio process started");
        }

        if let Some(ffmpeg_audio_stdin) = &self.ffmpeg_audio_stdin {
            let mut audio_stdin_lock = ffmpeg_audio_stdin.lock().await;
            *audio_stdin_lock = audio_stdin;
            drop(audio_stdin_lock);
            println!("Audio stdin set");
        }

        if custom_device != Some("None") {
            println!("Starting audio channel senders...");
            tokio::spawn(async move {
                while let Some(bytes) = &audio_channel_receiver
                    .lock()
                    .await
                    .as_mut()
                    .unwrap()
                    .recv()
                    .await
                {
                    if let Some(audio_stdin_arc) = &ffmpeg_audio_stdin {
                        let mut audio_stdin_guard = audio_stdin_arc.lock().await;
                        if let Some(ref mut stdin) = *audio_stdin_guard {
                            stdin
                                .write_all(&bytes)
                                .await
                                .expect("Failed to write audio data to FFmpeg stdin");
                        }
                        drop(audio_stdin_guard);
                    }
                }
            });
        }

        if custom_device != Some("None") {
            self.ffmpeg_audio_process = audio_child;
        }

        self.start_time = Some(Instant::now());
        self.audio_file_path = Some(audio_file_path_owned);
        self.device_name = Some(device.name().expect("Failed to get device name"));

        println!("End of the start_audio_recording function");

        Ok(())
    }

    pub fn trigger_play(&mut self) -> Result<(), &'static str> {
        if let Some(ref mut stream) = self.stream {
            stream.play().map_err(|_| "Failed to play stream")?;
            println!("Audio recording playing.");
        } else {
            return Err("Starting the recording did not work");
        }

        Ok(())
    }

    pub async fn stop_media_recording(&mut self) -> Result<(), String> {
        if let Some(start_time) = self.start_time {
            let segment_duration = Duration::from_secs(3);
            let recording_duration = start_time.elapsed();
            let expected_segments = recording_duration.as_secs() / segment_duration.as_secs();
            let audio_file_path = self
                .audio_file_path
                .as_ref()
                .ok_or("Audio file path not set")?;
            let audio_segment_list_filename = format!("{}/segment_list.txt", audio_file_path);

            loop {
                let audio_segments =
                    std::fs::read_to_string(&audio_segment_list_filename).unwrap_or_default();

                let audio_segment_count = audio_segments.lines().count();

                if audio_segment_count >= expected_segments as usize {
                    println!("All segments generated");
                    break;
                }

                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }

        if let Some(ref ffmpeg_audio_stdin) = self.ffmpeg_audio_stdin {
            let mut audio_stdin_guard = ffmpeg_audio_stdin.lock().await;
            if let Some(mut audio_stdin) = audio_stdin_guard.take() {
                if let Err(e) = audio_stdin.write_all(b"q\n").await {
                    eprintln!("Failed to send 'q' to audio FFmpeg process: {}", e);
                }
                let _ = audio_stdin.shutdown().await.map_err(|e| e.to_string());
            }
        }

        self.should_stop.store(true, Ordering::SeqCst);

        if let Some(sender) = self.audio_channel_sender.take() {
            drop(sender);
        }

        if let Some(ref mut stream) = self.stream {
            stream.pause().map_err(|_| "Failed to pause stream")?;
            println!("Audio recording paused.");
        } else {
            return Err("Original recording was not started".to_string());
        }

        if let Some(process) = &mut self.ffmpeg_audio_process {
            let _ = process.kill().await.map_err(|e| e.to_string());
        }

        println!("Audio recording stopped.");
        Ok(())
    }

    async fn start_audio_ffmpeg_processes(
        &self,
        ffmpeg_binary_path: &str,
        audio_ffmpeg_command: &[String],
    ) -> Result<(Child, ChildStdin), Error> {
        let mut audio_process = start_recording_process(ffmpeg_binary_path, audio_ffmpeg_command)
            .await
            .map_err(|e| {
                eprintln!("Failed to start audio recording process: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;

        let audio_stdin = audio_process.stdin.take().ok_or_else(|| {
            eprintln!("Failed to take audio stdin");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to take audio stdin")
        })?;

        Ok((audio_process, audio_stdin))
    }
}

#[tauri::command]
pub fn enumerate_audio_devices() -> Vec<String> {
    let host = cpal::default_host();
    let default_device = host
        .default_input_device()
        .expect("No default input device available");
    let default_device_name = default_device
        .name()
        .expect("Failed to get default device name");

    let devices = host.devices().expect("Failed to get devices");
    let mut input_device_names: Vec<String> = devices
        .filter_map(|device| {
            let supported_input_configs = device.supported_input_configs();
            if supported_input_configs.is_ok() && supported_input_configs.unwrap().count() > 0 {
                device.name().ok()
            } else {
                None
            }
        })
        .collect();

    input_device_names.retain(|name| name != &default_device_name);
    input_device_names.insert(0, default_device_name);

    input_device_names
}

use tokio::io::{AsyncBufReadExt, BufReader};

async fn start_recording_process(
    ffmpeg_binary_path_str: &str,
    args: &[String],
) -> Result<tokio::process::Child, std::io::Error> {
    let mut process = Command::new(ffmpeg_binary_path_str)
        .args(args)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(process_stderr) = process.stderr.take() {
        tokio::spawn(async move {
            let mut process_reader = BufReader::new(process_stderr).lines();
            while let Ok(Some(line)) = process_reader.next_line().await {
                eprintln!("FFmpeg process STDERR: {}", line);
            }
        });
    }

    Ok(process)
}

async fn wait_for_start_times(audio_start_time: Arc<Mutex<Option<Instant>>>) -> (Instant) {
    loop {
        let audio_start_locked = audio_start_time.lock().await;

        if audio_start_locked.is_some() {
            let audio_start = *audio_start_locked.as_ref().unwrap();
            return audio_start;
        }
        drop(audio_start_locked);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
