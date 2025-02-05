use crate::state;
use tauri::{AppHandle, Runtime, WebviewWindow};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn update_workspace_state<R: Runtime>(
    app_handle: AppHandle<R>,
    window: WebviewWindow<R>,
    patch: String,
) -> eyre::Result<()> {
    state::update_workspace_state(&app_handle, &window, patch).await
}
