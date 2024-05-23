use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

pub async fn summarize(text: String) -> Result<String, String> {
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
