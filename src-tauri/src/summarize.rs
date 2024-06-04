use std::{fs::File, io::Write, path::PathBuf};

use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SummaryJSON {
    pub result: String,
    pub action_items: String,
}

pub async fn summarize_and_write(
    text: String,
    summary_output_file_path: &PathBuf,
) -> Result<(), String> {
    let summary = summarize(&text).await?;
    let action_items = generate_action_items(&text).await?;

    let summary = SummaryJSON {
        result: summary,
        action_items,
    };

    let json_string =
        serde_json::to_string_pretty(&summary).expect("failed to serialize transcription");

    let mut file = File::create(summary_output_file_path).expect("couldn't create file");
    file.write_all(json_string.as_bytes())
        .expect("could not write to file");

    Ok(())
}

pub async fn summarize(text: &String) -> Result<String, String> {
    let ollama = Ollama::default();

    let model = "llama3:latest".to_string();
    let prompt = format!("Can you summarize this: {}", text);

    let res = ollama
        .generate(GenerationRequest::new(model, prompt))
        .await
        .expect("Failed to generate summary");

    Ok(res.response)
}

pub async fn generate_action_items(text: &String) -> Result<String, String> {
    let ollama = Ollama::default();

    let model = "llama3:latest".to_string();
    let prompt = format!("Can you create action items from this transcript: {}", text);

    let res = ollama
        .generate(GenerationRequest::new(model, prompt))
        .await
        .expect("failed to generate action items");

    Ok(res.response)
}

pub async fn generate_title(text: &String) -> Result<String, String> {
    let ollama = Ollama::default();

    let model = "llama3:latest".to_string();
    let prompt = format!("Can you generate a short meeting title from this: {}", text);

    let res = ollama
        .generate(GenerationRequest::new(model, prompt))
        .await
        .expect("failed to generate title");

    Ok(res.response)
}
