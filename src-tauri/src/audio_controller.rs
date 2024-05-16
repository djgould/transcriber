use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use hound::{WavSpec, WavWriter};

use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

struct Recorder {
    writer: Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    stream: Option<Stream>,
}

impl Recorder {
    fn new() -> Result<Self> {
        let writer = Arc::new(Mutex::new(None));

        Ok(Self {
            writer: writer,
            stream: None,
        })
    }

    fn start(&mut self) -> Result<()> {
        let device = get_device_by_id("Platy Microphone").expect("No input device available");
        let config = device.default_input_config()?;

        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        self.writer = Arc::new(Mutex::new(Some(WavWriter::create("output.wav", spec)?)));

        let writer_clone = self.writer.clone();
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut writer_lock) = writer_clone.lock() {
                    if let Some(ref mut writer) = *writer_lock {
                        for &sample in data {
                            let amplitude = (sample * i16::MAX as f32) as i16;
                            writer
                                .write_sample(amplitude)
                                .expect("Failed to write sample");
                        }
                    }
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
}

impl AudioController {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut recorder = Recorder::new().expect("Failed to initialize the recorder");
            for command in receiver {
                match command {
                    AudioCommand::Start => {
                        recorder.start().expect("Failed to start recording");
                    }
                    AudioCommand::Stop => {
                        recorder.stop().expect("Failed to stop recording");
                    }
                }
            }
        });
        AudioController { sender }
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
