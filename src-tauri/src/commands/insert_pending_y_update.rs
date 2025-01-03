use crate::database::query::insert;
use crate::types::util::{BytesBase64URL, UUIDv7Base64URL};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_pending_y_update<R: Runtime>(
    app_handle: AppHandle<R>,
    y_doc_id: UUIDv7Base64URL,
    y_update: BytesBase64URL,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    insert::from_local::pending_y_update(pool, y_doc_id, &y_update).await?;

    Ok(())
}
