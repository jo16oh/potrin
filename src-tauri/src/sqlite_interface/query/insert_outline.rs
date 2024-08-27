use super::super::POOL;
use crate::utils::get_once_lock;
use chrono::Utc;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline(text: &str, parent: Option<Vec<u8>>) -> anyhow::Result<Vec<u8>> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().into_bytes();
    let now = Utc::now().timestamp_millis();
    sqlx::query!(
        "INSERT INTO outlines (id, parent, text, created_at, updated_at, origin) VALUES (?, ?, ?, ?, ?, ?);",
        id,
        parent,
        text,
        now,
        now,
        1
    )
    .execute(pool)
    .await?;

    Ok(id)
}
