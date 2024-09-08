use super::super::POOL;
use crate::{utils::get_once_lock, OutlinesTable};
use anyhow::anyhow;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_outline(id: Vec<u8>) -> anyhow::Result<OutlinesTable> {
    let pool = get_once_lock(&POOL)?;
    sqlx::query_as!(OutlinesTable, r#"SELECT id , author, pot_id, parent_id, fractional_index, text, last_materialized_hash, created_at, updated_at, is_deleted FROM outlines WHERE id = ?;"#, id)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!(e.to_string()))
}
