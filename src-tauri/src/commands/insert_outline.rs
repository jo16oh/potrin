use crate::database::query;
use crate::types::model::{Outline, OutlineChangeEvent, YUpdate};
use crate::types::state::AppState;
use crate::types::util::{Operation, Origin};
use crate::utils::{get_rw_state, get_state};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
    y_updates: Vec<YUpdate>,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let mut tx = pool.begin().await?;

    let lock = get_rw_state::<R, AppState>(&app_handle)?;

    let app_state = lock.read().await;
    let pot_id = &app_state
        .pot
        .as_ref()
        .ok_or(anyhow!("pot state is not set"))?
        .id;

    query::insert_outline_y_updates(&mut *tx, &outline.id, &y_updates).await?;
    query::insert_outline(&mut *tx, &outline, pot_id).await?;

    tx.commit().await?;

    OutlineChangeEvent::new(Operation::Insert, Origin::Local, &[outline.clone()])
        .emit(&app_handle)?;

    Ok(())
}
