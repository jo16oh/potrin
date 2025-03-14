use crate::events::{AppStateChange, WorkspaceStateChange};
use crate::search_engine::{load_index, SearchIndex};
use crate::types::model::Pot;
use crate::types::setting::SearchFuzziness;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{get_rw_state, write};
use crate::{types::state::*, utils::set_rw_state};
use derive_more::derive::{Deref, DerefMut};
use eyre::OptionExt;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use tauri::{AppHandle, EventTarget, Manager, Runtime, WebviewWindow};
use tauri_specta::Event;
use tokio::sync::RwLock;

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

pub async fn init_app_state<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let path = app_handle
        .path()
        .app_data_dir()?
        .join("state")
        .join("app.json");

    let app_state: AppState = if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        let app_state = AppState::default();
        let json = serde_json::to_string_pretty(&app_state)?;

        write(&path, json)?;
        app_state
    };

    set_rw_state::<R, AppState>(app_handle, app_state).await?;

    Ok(())
}

pub async fn init_window_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot: &Pot,
) -> eyre::Result<()> {
    init_workspace_state(app_handle, pot).await?;
    init_search_engine(app_handle, pot).await?;

    Ok(())
}

async fn init_workspace_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot: &Pot,
) -> eyre::Result<()> {
    let path = app_handle
        .path()
        .app_data_dir()?
        .join("state")
        .join(pot.id.to_string())
        .join("workspace.json");

    let workspace_state: WorkspaceState = if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        let workspace_state = WorkspaceState::new(pot);
        let json = serde_json::to_string_pretty(&workspace_state)?;

        write(&path, json)?;
        workspace_state
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

    Ok(())
}

async fn init_search_engine<R: Runtime>(app_handle: &AppHandle<R>, pot: &Pot) -> eyre::Result<()> {
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
    current_app_state: AppState,
    origin_window_label: &str,
) -> eyre::Result<()> {
    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let path = app_handle
        .path()
        .app_data_dir()?
        .join("state")
        .join("app.json");

    let mut app_state = lock.write().await;

    *app_state = current_app_state;

    let json = serde_json::to_string_pretty(&*app_state)?;
    write(&path, json)?;

    AppStateChange::new(app_state.clone()).emit_filter(app_handle, |target| match target {
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
    current_workspace_state: WorkspaceState,
    emit_event: bool,
) -> eyre::Result<()> {
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;

    let path = app_handle
        .path()
        .app_data_dir()?
        .join("state")
        .join(pot_id.to_string())
        .join("workspace.json");

    let workspaces_lock = get_rw_state::<R, Workspaces>(window)?;
    let mut workspaces = workspaces_lock.write().await;

    let workspace_lock = workspaces
        .get_mut(&pot_id)
        .ok_or_eyre("workspace state is not set")?;
    let mut workspace = workspace_lock.write().await;

    *workspace = current_workspace_state;

    let json = serde_json::to_string_pretty(&*workspace)?;
    write(&path, json)?;

    if emit_event {
        WorkspaceStateChange::new(workspace.clone()).emit_filter(
            app_handle,
            |target| match target {
                EventTarget::WebviewWindow { label } => label == window.label(),
                EventTarget::Webview { label } => label == window.label(),
                EventTarget::Window { label } => label == window.label(),
                _ => false,
            },
        )?;
    }

    Ok(())
}

pub async fn close_pot<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: &UUIDv7Base64URL,
) -> eyre::Result<()> {
    let path = app_handle
        .path()
        .app_data_dir()?
        .join("state")
        .join("app.json");

    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let mut app_state = lock.write().await;
    app_state.pots.remove(pot_id);

    let json = serde_json::to_string_pretty(&*app_state)?;
    write(&path, json)?;

    let workspaces_lock = get_rw_state::<R, Workspaces>(app_handle)?;
    let mut workspaces = workspaces_lock.write().await;
    workspaces.remove(pot_id);

    let search_indices_lock = get_rw_state::<R, SearchIndices>(app_handle)?;
    let mut search_indices = search_indices_lock.write().await;
    search_indices.remove(pot_id);

    AppStateChange::new(app_state.clone()).emit(app_handle)?;
    Ok(())
}
