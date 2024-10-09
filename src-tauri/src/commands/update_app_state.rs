use crate::state::{self, AppStateValues};
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn update_app_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: AppStateValues,
) -> anyhow::Result<()> {
    state::update_app_state(app_handle, value).await
}
