use crate::types::model::Pot;
use crate::{database, utils::get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_pot<R: Runtime>(app_handle: AppHandle<R>, pot: Pot) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    database::query::insert_pot(pool, &pot).await
}
