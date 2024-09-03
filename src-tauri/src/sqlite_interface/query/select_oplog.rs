use super::super::table::OplogTable;
use super::super::POOL;
use crate::utils::get_once_lock;
use anyhow::anyhow;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_oplog(id: Vec<u8>) -> anyhow::Result<OplogTable> {
    let pool = get_once_lock(&POOL)?;
    sqlx::query_as!(
        OplogTable,
        r#"SELECT * FROM oplog WHERE primary_key = ?;"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))
}
