use std::{fs::File, io::Write, path::PathBuf};

use log::info;
use ollama_rs::{
    generation::{completion::request::GenerationRequest, parameters::FormatType},
    Ollama,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SummaryJSON {
    pub result: String,
    pub action_items: Vec<ActionItem>,
}

pub async fn summarize_and_write(
    text: String,
    summary_output_file_path: &PathBuf,
) -> Result<(), String> {
    let summary = summarize(&text).await?;
    let action_items = generate_action_items(&text).await?;

    let summary = SummaryJSON {
        result: summary,
        action_items: action_items.action_items,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionItem {
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionItems {
    action_items: Vec<ActionItem>,
}

pub async fn generate_action_items(text: &String) -> Result<ActionItems, String> {
    let ollama = Ollama::default();

    let model = "llama3:latest".to_string();
    let prompt = format!(
        "Create action items from a transcript.
        You must format your output as a JSON value that adheres to a given \"JSON Schema\" instance.
        \"JSON Schema\" is a declarative language that allows you to annotate and validate JSON documents.
        For example, the example \"JSON Schema\" instance {{\"properties\": {{\"foo\": {{\"description\": \"a list of test words\", \"type\": \"array\", \"items\": {{\"type\": \"string\"}}}}}}, \"required\": [\"foo\"]}}
        would match an object with one required property, \"foo\". The \"type\" property specifies \"foo\" must be an \"array\", and the \"description\" property semantically describes it as \"a list of test words\". The items within \"foo\" must be strings.
        Thus, the object {{\"foo\": [\"bar\", \"baz\"]}} is a well-formatted instance of this example \"JSON Schema\". The object {{\"properties\": {{\"foo\": [\"bar\", \"baz\"]}}}} is not well-formatted.
        Your output will be parsed and type-checked according to the provided schema instance, so make sure all fields in your output match the schema exactly and there are no trailing commas!
        Here is the JSON Schema instance your output must adhere to. Include the enclosing markdown codeblock:
        ```json
        {{
            \"type\": \"object\",
            \"properties\": {{
                \"action_items\": {{
                    \"type\": \"array\",
                    \"items\": {{
                        \"type\": \"object\",
                        \"properties\": {{
                            \"title\": {{
                                \"type\": \"string\",
                                \"description\": \"The title of the action item\"
                            }}
                        }},
                        \"required\": [\"title\"],
                        \"additionalProperties\": false
                    }}
                }}
            }},
            \"required\": [\"actionItems\"],
            \"additionalProperties\": false,
            \"$schema\": \"http://json-schema.org/draft-07/schema#\"
        }}
        ```
        transcript: {}",
        text
    );

    let generation_request = GenerationRequest::new(model, prompt).format(FormatType::Json);

    let res = ollama
        .generate(generation_request)
        .await
        .expect("failed to generate action items");
    info!("action items: {}", res.response);
    let json: ActionItems =
        serde_json::from_str(&res.response).expect("Action items not formatted correctly");
    Ok(json)
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
