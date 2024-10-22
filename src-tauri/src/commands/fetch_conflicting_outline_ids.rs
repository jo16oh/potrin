use crate::database::query;
use crate::types::util::Base64;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_conflicting_outline_ids<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<Base64>,
) -> anyhow::Result<Vec<Base64>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    query::fetch_conflictint_outline_ids(pool, outline_ids.iter().collect()).await
}
