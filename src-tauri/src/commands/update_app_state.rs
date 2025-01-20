use crate::state;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn update_app_state<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    patch: String,
) -> eyre::Result<()> {
    state::update_app_state(&app_handle, patch, window.label()).await
}
