use crate::{
    state::Workspaces,
    types::{state::WorkspaceState, util::UUIDv7Base64URL},
    utils::get_rw_state,
};
use eyre::OptionExt;
use tauri::Window;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn get_workspace_state(window: Window) -> eyre::Result<WorkspaceState> {
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;
    let workspaces_lock = get_rw_state::<_, Workspaces>(&window)?;
    let workspaces = workspaces_lock.write().await;
    let workspace_lock = workspaces
        .get(&pot_id)
        .ok_or_eyre("workspace state is not set")?;
    let workspace = workspace_lock.read().await.clone();

    eyre::Ok(workspace)
}
