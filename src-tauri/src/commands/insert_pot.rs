use crate::database;
use crate::types::model::Pot;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_pot<R: Runtime>(app_handle: AppHandle<R>, pot: Pot) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    database::query::insert_pot(pool, &pot).await
}
