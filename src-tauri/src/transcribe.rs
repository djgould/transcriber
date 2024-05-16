use std::{sync::Arc, thread};

use anyhow::Error;
use flume::{bounded, unbounded, Receiver, Sender};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

enum TranscriberCommand {
    Data(Vec<f32>),
}
pub struct TranscriberController {
    sender: Sender<TranscriberCommand>,
}

impl TranscriberController {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(100);
        thread::spawn(move || {
            let transcriber = Transcriber::new().expect("Failed to initialize the recorder");
            transcriber.run();
            for command in receiver {
                match command {
                    TranscriberCommand::Data(audio_data) => {
                        transcriber
                            .add_chunk(audio_data)
                            .expect("Failed to start recording");
                    }
                }
            }
        });
        TranscriberController { sender }
    }

    pub fn add_chunk(&self, audio_data: Vec<f32>) {
        self.sender
            .send(TranscriberCommand::Data(audio_data))
            .expect("Failed to send start command");
    }
}

struct Transcriber {
    sender: Arc<Sender<Vec<f32>>>,
    receiver: Arc<Receiver<Vec<f32>>>,
}

impl Transcriber {
    pub fn new() -> Result<Self, Error> {
        let (sender, receiver) = unbounded();
        Ok(Self {
            sender: Arc::new(sender),
            receiver: Arc::new(receiver),
        })
    }

    pub fn run(&self) {
        self.transcribe(self.receiver.clone());
    }

    pub fn add_chunk(&self, audio_data: Vec<f32>) -> Result<(), String> {
        println!("got chunk");
        self.sender
            .send(audio_data)
            .map_err(|_| "Failed to send data to transcriber")?;
        Ok(())
    }

    fn transcribe(&self, rx: Arc<Receiver<Vec<f32>>>) {
        thread::spawn(move || {
            use std::path::Path;

            let whisper_path =
                Path::new("/Users/devingould/platy/src-tauri/src/models/ggml-small.en-tdrz.bin");
            if !whisper_path.exists() {
                panic!("whisper file doesn't exist");
            }

            let ctx: WhisperContext = WhisperContext::new_with_params(
                &whisper_path.to_string_lossy(),
                WhisperContextParameters::default(),
            )
            .expect("failed to open model");
            let mut state = ctx.create_state().expect("failed to create state");
            let mut sample_old: Vec<f32> = Vec::new();
            let step_size: usize = 3000; // in milliseconds
            let sample_rate: usize = 16000;
            let n_samples_step: usize = step_size * sample_rate / 1000;
            let n_samples_keep: usize = 200; // in milliseconds
            let n_samples_keep: usize = n_samples_keep * sample_rate / 1000;
            println!("transcriber starting");
            while let Ok(sample_new) = rx.recv() {
                println!("got new sample");
                let n_samples_new = sample_new.len();
                let n_samples_take = std::cmp::min(
                    sample_old.len(),
                    n_samples_keep
                        .saturating_add(n_samples_step)
                        .saturating_sub(n_samples_new),
                );

                let mut sample: Vec<f32> = vec![0.0; n_samples_new + n_samples_take];

                for i in 0..n_samples_take {
                    sample[i] = sample_old[sample_old.len() - n_samples_take + i];
                }

                let mut params = FullParams::new(SamplingStrategy::default());
                params.set_initial_prompt("experience");
                params.set_progress_callback_safe(|progress| {
                    println!("Progress callback: {}%", progress)
                });
                // params.set_tdrz_enable(true);
                // convert to float (f32

                let st = std::time::Instant::now();

                state
                    .full(params, &sample)
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
                println!("{}", full_text.join("\n"));
                println!("{}", num_segments);
                println!("Transcription took {}ms", (et - st).as_millis());
            }
        });
    }
}
