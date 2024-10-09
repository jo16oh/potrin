// use super::super::table::OplogTable;
// use anyhow::anyhow;
// use sqlx::SqlitePool;
// use tauri::{AppHandle, Manager, Runtime};
//
// #[tauri::command]
// #[specta::specta]
// #[macros::anyhow_to_string]
// pub async fn select_oplog<R: Runtime>(
//     app_handle: AppHandle<R>,
//     id: Vec<u8>,
// ) -> anyhow::Result<OplogTable> {
//     let pool = app_handle
//         .try_state::<SqlitePool>()
//         .ok_or(anyhow!("failed to get SqlitePool"))?
//         .inner();
//
//     sqlx::query_as!(
//         OplogTable,
//         r#"SELECT * FROM oplog WHERE primary_key = ?;"#,
//         id
//     )
//     .fetch_one(pool)
//     .await
//     .map_err(|e| anyhow!(e.to_string()))
// }
