use crate::types::model::Pot;
use crate::{database::query, utils::get_state};
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn create_pot<R: Runtime>(app_handle: AppHandle<R>, pot: Pot) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let now = Utc::now().timestamp_millis();

    query::insert::from_local::pot(pool, &pot, now).await?;

    eyre::Ok(())
}
