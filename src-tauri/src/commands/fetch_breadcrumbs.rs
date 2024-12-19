use crate::database::query::fetch;
use crate::types::model::Breadcrumbs;
use crate::types::util::UUIDv7Base64;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_breadcrumbs<R: Runtime>(
    app_handle: AppHandle<R>,
    parent_id: UUIDv7Base64,
) -> anyhow::Result<Breadcrumbs> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::breadcrumbs(pool, parent_id).await
}
