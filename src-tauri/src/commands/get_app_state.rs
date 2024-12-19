use tauri::{async_runtime::RwLock, State};
use crate::types::state::AppState;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn get_app_state(app_state: State<'_, RwLock<AppState>>) -> anyhow::Result<AppState> {
  Ok(app_state.read().await.clone())
}
