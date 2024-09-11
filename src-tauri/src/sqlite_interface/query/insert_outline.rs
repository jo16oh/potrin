use super::super::table::{
    types::Base64String, types::Operation::*, types::Origin, OutlinesTable,
    OutlinesTableChangeEvent,
};
use crate::types::NullableBase64String;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    text: Option<&str>,
    parent: NullableBase64String,
    origin: Origin,
) -> anyhow::Result<OutlinesTable> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let id = Base64String::from_bytes(uuidv7::create().into_bytes());

    let outline: OutlinesTable = sqlx::query_as!(
        OutlinesTable,
        r#"
            INSERT INTO outlines (id, parent_id, fractional_index, text)
            VALUES (?, ?, ?, ?)
            RETURNING *;"#,
        id,
        parent,
        "",
        text,
    )
    .fetch_one(pool)
    .await?;

    OutlinesTableChangeEvent::new(Insert, origin, &[outline.clone()]).emit(&app_handle)?;

    Ok(outline)
}
