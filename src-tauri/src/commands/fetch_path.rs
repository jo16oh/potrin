use crate::database::query::fetch;
use crate::types::model::Path;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_path<R: Runtime>(
    app_handle: AppHandle<R>,
    parent_id: UUIDv7Base64URL,
) -> anyhow::Result<Path> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::path(pool, parent_id).await
}
