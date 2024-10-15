use crate::database::query;
use crate::types::model::{Outline, OutlineChangeEvent};
use crate::types::util::{Operation, Origin};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn delete_outline_physically<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    query::delete_outline_physically(pool, &outline.id).await?;

    OutlineChangeEvent::new(Operation::Delete, Origin::Local, &[outline]).emit(&app_handle)?;

    Ok(())
}
