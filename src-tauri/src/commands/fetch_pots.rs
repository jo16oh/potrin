use crate::database::query::fetch;
use crate::types::model::Pot;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn fetch_pots<R: Runtime>(app_handle: AppHandle<R>) -> eyre::Result<Vec<Pot>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    fetch::pots(pool).await
}
