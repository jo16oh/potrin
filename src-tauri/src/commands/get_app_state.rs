use crate::types::state::AppState;
use tauri::{async_runtime::RwLock, State};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn get_app_state(app_state: State<'_, RwLock<AppState>>) -> eyre::Result<AppState> {
    eyre::Ok(app_state.read().await.clone())
}
