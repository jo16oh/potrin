use crate::{types::state::WorkspaceState, utils::get_rw_state};
use tauri::Window;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn get_workspace_state(window: Window) -> eyre::Result<WorkspaceState> {
    let state = get_rw_state::<_, WorkspaceState>(&window)?
        .read()
        .await
        .clone();

    eyre::Ok(state)
}
