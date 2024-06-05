use crate::recorder::RecordingState;
use std::sync::Arc;
use tauri::{async_runtime::Mutex, State};

#[tauri::command]
pub async fn is_recording(state: State<'_, Arc<Mutex<RecordingState>>>) -> Result<bool, String> {
    let guard = state.lock().await;

    Ok(guard.media_process.is_some())
}
