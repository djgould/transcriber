use std::fs::read_to_string;
use std::sync::Arc;

use entity::conversation::{self, Model as ConversationModel};
use log::info;
use service::{
    sea_orm::{DeleteResult, TryIntoModel},
    Mutation, Query,
};

use crate::{recorder::RecordingState, summarize::SummaryJSON, AppState};

#[tauri::command]
pub async fn get_conversation(
    state: tauri::State<'_, AppState>,
    conversation_id: i32,
) -> Result<conversation::Model, ()> {
    let result: Option<ConversationModel> =
        Query::find_conversation_by_id(&state.db, conversation_id)
            .await
            .expect("Cannot find posts in page");

    let conversation = match result {
        Some(conversation) => {
            // Use your conversation model here
            conversation
        }
        None => {
            // Handle the case where conversation was not found
            eprintln!("Conversation not found");
            return Err(());
        }
    };

    Ok(conversation)
}

#[tauri::command]
pub async fn get_conversations(
    state: tauri::State<'_, AppState>,
    page: u64,
    items_per_page: u64,
) -> Result<Vec<conversation::Model>, ()> {
    info!("getting conversations...");
    let (conversations, num_pages) =
        Query::find_conversations_in_page(&state.db, page, items_per_page)
            .await
            .expect("Cannot find posts in page");

    Ok(conversations)
}

#[tauri::command]
pub async fn create_conversation(
    state: tauri::State<'_, AppState>,
    form: conversation::Model,
) -> Result<conversation::Model, String> {
    let _ = &state.db;

    let conversation = Mutation::create_conversation(&state.db, form)
        .await
        .expect("could not insert post");

    Ok(conversation
        .try_into_model()
        .expect("could not turn result into a model"))
}

#[tauri::command]
pub async fn delete_conversation(
    state: tauri::State<'_, AppState>,
    conversation_id: i32,
) -> Result<u64, String> {
    let _ = &state.db;

    let result = Mutation::delete_conversation(&state.db, conversation_id)
        .await
        .expect("could not delete conversation");

    Ok(result.rows_affected)
}

#[tauri::command]
pub async fn get_summary_for_converstation(
    state: tauri::State<'_, Arc<tauri::async_runtime::Mutex<RecordingState>>>,
    conversation_id: i32,
) -> Result<SummaryJSON, String> {
    let state_guard = state.lock().await;
    let data_dir = match &state_guard.data_dir {
        Some(dir) => dir,
        None => return Err("Data directory not set".to_string()),
    };

    let path = data_dir
        .join("chunks/audio")
        .join(conversation_id.to_string())
        .join("summary.json");

    let content = read_to_string(&path)
        .map_err(|err| format!("Failed to read file {}: {}", path.display(), err))?;

    let json_content: SummaryJSON = serde_json::from_str(&content)
        .map_err(|err| format!("Failed to parse JSON in file {}: {}", path.display(), err))?;

    Ok(json_content)
}

use tauri::Manager;

#[tauri::command]
pub async fn open_conversation(
    app_handle: tauri::AppHandle,
    conversation_id: u32,
) -> Result<(), String> {
    let window = app_handle.get_webview_window("app-window");
    if window.is_none() {
        println!("No window found");
        let mut window = tauri::WebviewWindowBuilder::from_config(
            &app_handle,
            &app_handle.config().app.windows.get(1).unwrap().clone(),
        )
        .unwrap()
        .build()
        .expect("Failed to create window");
        let mut new_url = window.url().unwrap();
        new_url.set_path(&format!("main/conversations/{}", conversation_id));
        window.navigate(new_url);
    } else {
        let mut window = window.unwrap();
        let mut new_url = window.url().unwrap();
        new_url.set_path(&format!("main/conversations/{}", conversation_id));
        window.navigate(new_url);

        let _ = window.set_focus();
    }

    Ok(())
}
