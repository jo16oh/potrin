use crate::{types::state::WorkspaceState, utils::get_rw_state};
use tauri::Window;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn get_workspace_state(window: Window) -> anyhow::Result<WorkspaceState> {
    let state = get_rw_state::<_, WorkspaceState>(&window)?
        .read()
        .await
        .clone();

    Ok(state)
}
