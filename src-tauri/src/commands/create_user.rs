use crate::types::model::User;
use crate::{database, utils::get_state};
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn create_user<R: Runtime>(app_handle: AppHandle<R>, user: User) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let now = Utc::now().timestamp_millis();

    database::query::insert::from_local::user(pool, &user, now).await
}
