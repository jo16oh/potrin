use crate::database::query;
use crate::types::model::YUpdate;
use crate::types::util::Base64;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_card_y_updates<R: Runtime>(
    app_handle: AppHandle<R>,
    id: Base64,
) -> anyhow::Result<Vec<YUpdate>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    query::fetch_card_y_updates_by_card_id(pool, &id).await
}
