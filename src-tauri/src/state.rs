use crate::database::query::{fetch, upsert};
use crate::events::AppStateChange;
use crate::search_engine::{load_index, SearchIndex};
use crate::types::model::Pot;
use crate::types::setting::SearchFuzziness;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{get_rw_state, get_state};
use crate::{types::state::*, utils::set_rw_state};
use derive_more::derive::{Deref, DerefMut};
use eyre::OptionExt;
use json_patch::Patch;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tauri::{AppHandle, EventTarget, Runtime, WebviewWindow};
use tauri_specta::Event;
use tokio::sync::RwLock;

pub async fn init_app_state<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let app_state = match fetch::app_state(pool).await? {
        Some(state) => state,
        None => {
            let app_state = AppState::default();
            upsert::app_state(pool, &app_state).await?;
            app_state
        }
    };

    set_rw_state::<R, AppState>(app_handle, app_state).await?;

    Ok(())
}

#[derive(Deref, DerefMut)]
pub struct Workspaces(HashMap<UUIDv7Base64URL, RwLock<WorkspaceState>>);

impl Workspaces {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Deref, DerefMut)]
pub struct SearchIndices(HashMap<UUIDv7Base64URL, SearchIndex>);

impl SearchIndices {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

pub async fn init_window_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot: &Pot,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let workspace_state = match fetch::workspace_state(pool, pot.id).await? {
        Some(state) => state,
        None => {
            let workspace_state = WorkspaceState::new(pot);
            upsert::workspace_state(pool, &workspace_state).await?;
            workspace_state
        }
    };

    if let Ok(lock) = get_rw_state::<_, Workspaces>(app_handle) {
        let mut workspaces = lock.write().await;
        workspaces.insert(pot.id, RwLock::new(workspace_state));
    } else {
        set_rw_state::<_, Workspaces>(app_handle, Workspaces::new()).await?;
        let lock = get_rw_state::<_, Workspaces>(app_handle)?;
        let mut workspaces = lock.write().await;

        workspaces.insert(pot.id, RwLock::new(workspace_state));
    };

    let search_index = load_index(app_handle, pot.id, SearchFuzziness::Fuzzy).await?;

    if let Ok(lock) = get_rw_state::<_, SearchIndices>(app_handle) {
        let mut search_indices = lock.write().await;
        search_indices.insert(pot.id, search_index);
    } else {
        set_rw_state::<_, SearchIndices>(app_handle, SearchIndices::new()).await?;
        let lock = get_rw_state::<_, SearchIndices>(app_handle)?;
        let mut search_indices = lock.write().await;

        search_indices.insert(pot.id, search_index);
    };

    Ok(())
}

pub async fn update_app_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    patch: String,
    origin_window_label: &str,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let mut app_state = lock.write().await;
    let patch_deserealized = &serde_json::from_str::<Patch>(&patch)?;

    let current_app_state = {
        let mut value = serde_json::to_value(&*app_state)?;
        json_patch::patch(&mut value, patch_deserealized)?;
        serde_json::from_value::<AppState>(value)?
    };

    upsert::app_state(pool, &current_app_state).await?;

    *app_state = current_app_state;

    AppStateChange::new(patch).emit_filter(app_handle, |target| match target {
        EventTarget::WebviewWindow { label } => label != origin_window_label,
        EventTarget::Webview { label } => label != origin_window_label,
        EventTarget::Window { label } => label != origin_window_label,
        _ => true,
    })?;

    Ok(())
}

pub async fn update_workspace_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &WebviewWindow<R>,
    patch: String,
) -> eyre::Result<()> {
    let pot_id = window.label().try_into()?;
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let workspaces_lock = get_rw_state::<R, Workspaces>(window)?;
    let mut workspaces = workspaces_lock.write().await;

    let workspace_lock = workspaces
        .get_mut(&pot_id)
        .ok_or_eyre("workspace state is not set")?;
    let mut workspace = workspace_lock.write().await;

    let patch_serealized = &serde_json::from_str::<Patch>(&patch)?;

    let current_workspace_state = {
        let mut value = serde_json::to_value(&*workspace)?;
        json_patch::patch(&mut value, patch_serealized)?;
        serde_json::from_value::<WorkspaceState>(value)?
    };

    upsert::workspace_state(pool, &current_workspace_state).await?;

    *workspace = current_workspace_state;

    Ok(())
}

pub async fn close_pot<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: &UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let mut app_state = lock.write().await;
    let prev_app_state = serde_json::to_value(app_state.clone())?;
    app_state.pots.remove(pot_id);
    let current_app_state = serde_json::to_value(app_state.clone())?;

    let patch = json_patch::diff(&prev_app_state, &current_app_state).to_string();

    upsert::app_state(pool, &app_state).await?;

    let workspaces_lock = get_rw_state::<R, Workspaces>(app_handle)?;
    let mut workspaces = workspaces_lock.write().await;
    workspaces.remove(pot_id);

    let search_indices_lock = get_rw_state::<R, SearchIndices>(app_handle)?;
    let mut search_indices = search_indices_lock.write().await;
    search_indices.remove(pot_id);

    AppStateChange::new(patch).emit(app_handle)?;
    Ok(())
}
