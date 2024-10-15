use crate::database::query;
use crate::types::model::{Outline, OutlineChangeEvent, OutlineYUpdate};
use crate::types::state::AppState;
use crate::types::util::{Operation, Origin};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
    y_updates: Vec<OutlineYUpdate>,
) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get app state"))?
        .inner();

    let app_state = lock.read().await;
    let pot_id = &app_state
        .pot
        .as_ref()
        .ok_or(anyhow!("pot state is not set"))
        .unwrap()
        .id;

    query::insert_outline_y_updates(&mut *tx, &outline.id, &y_updates).await?;
    query::insert_outline(&mut *tx, &outline, pot_id).await?;

    tx.commit().await?;

    OutlineChangeEvent::new(Operation::Insert, Origin::Local, &[outline.clone()])
        .emit(&app_handle)?;

    Ok(())
}
