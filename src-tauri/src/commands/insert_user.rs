use crate::types::model::User;
use crate::{database, utils::get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_user<R: Runtime>(app_handle: AppHandle<R>, user: User) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    database::query::insert_user(pool, &user).await
}
