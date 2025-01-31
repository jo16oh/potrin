use crate::types::util::UUIDv7Base64URL;
use crate::{database::query, utils::get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn delete_pot<R: Runtime>(
    app_handle: AppHandle<R>,
    pot_id: UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    query::delete::pot(pool, pot_id).await?;

    eyre::Ok(())
}
