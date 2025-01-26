use crate::database::query::fetch;
use crate::types::model::Path;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_path<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_id: UUIDv7Base64URL,
) -> eyre::Result<Option<Path>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::path(pool, outline_id).await
}
