use super::super::table::OutlinesTable;
use super::super::POOL;
use crate::sqlite_interface::table::{Operation, TableChangeEvent};
use crate::utils::get_once_lock;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    text: Option<&str>,
    parent: Option<Vec<u8>>,
) -> anyhow::Result<OutlinesTable> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().into_bytes();

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

    TableChangeEvent::<OutlinesTable>::new(Operation::Insert, &[outline.clone()])
        .emit(&app_handle)?;

    Ok(outline)
}
