use crate::state;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn update_workspace_state<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    patch: String,
) -> eyre::Result<()> {
    state::update_workspace_state(&app_handle, &window, patch).await
}
