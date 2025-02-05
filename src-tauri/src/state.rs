use crate::database::query::{fetch, upsert};
use crate::events::{AppStateChange, WorkspaceStateChange};
use crate::search_engine::SearchIndex;
use crate::types::model::Pot;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{get_rw_state, get_state};
use crate::{types::state::*, utils::set_rw_state};
use json_patch::Patch;
use sqlx::SqlitePool;
use tauri::{AppHandle, EventTarget, Manager, Runtime, WebviewWindow, Window};
use tauri_specta::Event;

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

pub async fn init_workspace_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &WebviewWindow<R>,
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

    set_rw_state::<R, WorkspaceState>(window, workspace_state).await?;

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
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, WorkspaceState>(window)?;

    let mut workspace_state = lock.write().await;
    let patch_serealized = &serde_json::from_str::<Patch>(&patch)?;

    let current_workspace_state = {
        let mut value = serde_json::to_value(&*workspace_state)?;
        json_patch::patch(&mut value, patch_serealized)?;
        serde_json::from_value::<WorkspaceState>(value)?
    };

    upsert::workspace_state(pool, &current_workspace_state).await?;

    *workspace_state = current_workspace_state;

    WorkspaceStateChange::new(patch).emit_filter(app_handle, |target| match target {
        EventTarget::WebviewWindow { label } => label == window.label(),
        EventTarget::Webview { label } => label == window.label(),
        EventTarget::Window { label } => label == window.label(),
        _ => false,
    })?;

    Ok(())
}

pub async fn close_pot<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &Window<R>,
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

    window.unmanage::<SearchIndex>();

    AppStateChange::new(patch).emit(app_handle)?;
    Ok(())
}
