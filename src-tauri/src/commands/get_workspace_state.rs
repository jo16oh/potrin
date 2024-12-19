use crate::types::state::WorkspaceState;
use tauri::{async_runtime::RwLock, State};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn get_workspace_state(
    workspace_state: State<'_, RwLock<WorkspaceState>>,
) -> anyhow::Result<WorkspaceState> {
    Ok(workspace_state.read().await.clone())
}
