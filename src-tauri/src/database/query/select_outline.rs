use crate::database::table::OutlinesTable;
use crate::database::types::Base64String;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    id: Base64String,
) -> anyhow::Result<OutlinesTable> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    sqlx::query_as!(OutlinesTable, r#"
        SELECT id , author, pot_id, parent_id, fractional_index, text, last_materialized_hash, created_at, updated_at, is_deleted
        FROM outlines
        WHERE id = ?;
        "#, id)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!(e.to_string()))
}