use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use hound::{WavSpec, WavWriter};

use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::transcribe::{self, TranscriberController};

struct Recorder {
    writer: Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    stream: Option<Stream>,
}

// fn clean_and_create_dir(dir: &Path) -> Result<(), String> {
//     if dir.exists() {
//         // Instead of just reading the directory, this will also handle subdirectories.
//         std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
//     }
//     std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

//     if !dir.to_string_lossy().contains("screenshots") {
//         let segment_list_path = dir.join("segment_list.txt");
//         match File::open(&segment_list_path) {
//             Ok(_) => Ok(()),
//             Err(ref e) if e.kind() == ErrorKind::NotFound => {
//                 File::create(&segment_list_path).map_err(|e| e.to_string())?;
//                 Ok(())
//             }
//             Err(e) => Err(e.to_string()),
//         }
//     } else {
//         Ok(())
//     }
// }

impl Recorder {
    fn new() -> Result<Self> {
        let writer = Arc::new(Mutex::new(None));

        Ok(Self {
            writer: writer,
            stream: None,
        })
    }

    fn start(
        &mut self,
        transcriber_controller: &Arc<transcribe::TranscriberController>,
    ) -> Result<()> {
        let device = get_device_by_id("Platy Microphone").expect("No input device available");
        let config = device.default_input_config()?;

        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        self.writer = Arc::new(Mutex::new(Some(WavWriter::create("output.wav", spec)?)));
        let transcriber_clone = transcriber_controller.clone();
        let buffer_size = spec.sample_rate * spec.channels as u32 * 2; // 2 seconds worth of samples

        // Shared buffer and control flag between threads
        let buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(buffer_size as usize)));
        let buffer_clone = Arc::clone(&buffer);
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer_clone.lock().unwrap();

                // Accumulate samples in the buffer
                buffer.extend_from_slice(data);

                // If the buffer has reached the target size, process it and clear the buffer
                if buffer.len() >= buffer_size as usize {
                    let chunk: Vec<f32> = buffer.drain(..).collect();
                    transcriber_clone.add_chunk(chunk);
                }
            },
            |err| eprintln!("Error: {:?}", err),
            Some(std::time::Duration::from_secs(300)),
        )?;

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!("Stopping recording");
        if let Some(stream) = self.stream.take() {
            stream.pause()?;
            drop(stream);
        }

        // Take out the WavWriter, finalize it, and replace with None
        let maybe_writer = {
            let mut writer_lock = self.writer.lock().unwrap();
            writer_lock.take() // This takes the WavWriter out and leaves None in its place
        };

        if let Some(mut writer) = maybe_writer {
            writer.finalize()?; // Now you can finalize without moving out of the MutexGuard
        }

        Ok(())
    }
}

enum AudioCommand {
    Start,
    Stop,
}

fn get_device_by_id(device_id_str: &str) -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    // Enumerate the available input devices.
    let devices = host.input_devices()?;

    // Find the device that matches the provided id.
    for device in devices {
        println!("{}", device.name()?);
        if device.name()? == device_id_str {
            return Ok(device);
        }
    }

    Err(format!("No device found with id: {}", device_id_str).into())
}

pub struct AudioController {
    sender: Sender<AudioCommand>,
    transcriber_controller: Arc<TranscriberController>,
}

impl AudioController {
    pub fn new(transcriber_controller: &Arc<TranscriberController>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let transcriber_clone = transcriber_controller.clone();
        thread::spawn(move || {
            let mut recorder = Recorder::new().expect("Failed to initialize the recorder");
            for command in receiver {
                match command {
                    AudioCommand::Start => {
                        recorder
                            .start(&transcriber_clone)
                            .expect("Failed to start recording");
                    }
                    AudioCommand::Stop => {
                        recorder.stop().expect("Failed to stop recording");
                    }
                }
            }
        });
        AudioController {
            sender,
            transcriber_controller: transcriber_controller.clone(),
        }
    }

    pub fn start(&self) {
        self.sender
            .send(AudioCommand::Start)
            .expect("Failed to send start command");
    }

    pub fn stop(&self) {
        self.sender
            .send(AudioCommand::Stop)
            .expect("Failed to send stop command");
    }
}
