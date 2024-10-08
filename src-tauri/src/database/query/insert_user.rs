use crate::database::table::User;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_user<R: Runtime>(app_handle: AppHandle<R>, user: User) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    sqlx::query!(
        r#"
            INSERT INTO users (id, name)
            VALUES (?, ?);
        "#,
        user.id,
        user.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
