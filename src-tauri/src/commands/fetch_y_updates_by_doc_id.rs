use crate::database::query::fetch;
use crate::types::util::BytesBase64URL;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_y_updates_by_doc_id<R: Runtime>(
    app_handle: AppHandle<R>,
    y_doc_id: UUIDv7Base64URL,
) -> anyhow::Result<Vec<BytesBase64URL>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::y_updates_by_doc_id(pool, y_doc_id).await
}
