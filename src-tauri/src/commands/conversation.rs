use entity::conversation::{self, Model as ConversationModel};
use service::{
    sea_orm::{DeleteResult, TryIntoModel},
    Mutation, Query,
};

use crate::AppState;

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
) -> Result<Vec<conversation::Model>, ()> {
    let page = 1;
    let (conversations, num_pages) = Query::find_conversations_in_page(&state.db, page, 1000)
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
