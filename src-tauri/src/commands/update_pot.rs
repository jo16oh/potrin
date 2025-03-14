use std::fs::File;
use std::io::BufReader;

use crate::database::query::update;
use crate::events::WorkspaceStateChange;
use crate::state::{update_workspace_state, Workspaces};
use crate::types::model::Pot;
use crate::types::state::WorkspaceState;
use crate::utils::get_state;
use crate::utils::{get_rw_state, write};
use chrono::Utc;
use eyre::OptionExt;
use garde::Unvalidated;
use sqlx::SqlitePool;
use tauri::{AppHandle, EventTarget, Manager, Runtime, Window};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn update_pot<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    pot: Pot,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let unvalidated = Unvalidated::new(pot);
    let pot = unvalidated.validate()?;

    let now = Utc::now().timestamp_millis();

    update::pot(pool, &pot, now).await?;

    if let Some(win) = app_handle.get_webview_window(&pot.id.to_string()) {
        win.set_title(&pot.name)?;

        let workspaces_lock = get_rw_state::<_, Workspaces>(&window)?;
        let workspaces = workspaces_lock.write().await;
        let workspace_lock = workspaces
            .get(&pot.id)
            .ok_or_eyre("workspace state is not set")?;

        let prev = workspace_lock.read().await.clone();
        let current = {
            let mut c = prev.clone();
            c.pot = pot.into_inner();
            c
        };

        let diff = {
            let prev = serde_json::to_value(prev)?;
            let current = serde_json::to_value(current)?;
            json_patch::diff(&prev, &current).to_string()
        };

        drop(workspaces);

        update_workspace_state(&app_handle, &win, diff.clone()).await?;

        WorkspaceStateChange::new(diff).emit_filter(&app_handle, |target| match target {
            EventTarget::WebviewWindow { label } => label == window.label(),
            EventTarget::Webview { label } => label == window.label(),
            EventTarget::Window { label } => label == window.label(),
            _ => false,
        })?;
    } else {
        let path = app_handle
            .path()
            .app_data_dir()?
            .join("state")
            .join(pot.id.to_string())
            .join("workspace.json");

        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            let mut workspace: WorkspaceState = serde_json::from_reader(reader)?;
            workspace.pot = pot.into_inner();

            let json = serde_json::to_string_pretty(&workspace)?;

            write(&path, json)?;
        }
    }

    eyre::Ok(())
}
