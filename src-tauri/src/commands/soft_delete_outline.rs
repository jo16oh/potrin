use crate::database::query::delete;
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::Outline;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn soft_delete_outline<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    outline: Outline,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let rowids = delete::soft::outlines(pool, &[outline.id]).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    Ok(())
}
