use crate::{
    search_engine::load_index,
    state::{self, AppStateValues},
};
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn update_app_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: AppStateValues,
) -> anyhow::Result<()> {
    let pot_changed = matches!(value, AppStateValues::Pot(_));
    state::update_app_state(&app_handle, value).await?;

    if pot_changed {
        load_index(&app_handle, 0).await?;
    }

    Ok(())
}
