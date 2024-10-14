use crate::database::query;
use crate::types::model::{Outline, OutlineChangeEvent};
use crate::types::util::{Operation, Origin};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn delete_outline_physically<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    query::delete_outline_physically(pool, &outline.id).await?;

    OutlineChangeEvent::new(Operation::Delete, Origin::Local, &[outline]).emit(&app_handle)?;

    Ok(())
}
