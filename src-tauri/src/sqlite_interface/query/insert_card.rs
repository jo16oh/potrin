use super::super::table::{CardsTable, CardsTableChangeEvent, Operation};
use super::super::POOL;
use super::insert_outline;
use crate::utils::get_once_lock;
use anyhow::anyhow;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_card<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    text: &str,
    outline_id: Option<Vec<u8>>,
) -> anyhow::Result<CardsTable> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().into_bytes();

    let outline_id = match outline_id {
        Some(id) => id,
        None => insert_outline("", None)
            .await
            .map_err(|e| anyhow!(e.to_string()))?,
    };

    let card = sqlx::query_as!(
        CardsTable,
        r#"
            INSERT INTO cards (id, outline_id, fractional_index, text, from_remote)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *;
        "#,
        id,
        outline_id,
        "",
        text,
        1
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))?;

    let event = CardsTableChangeEvent::new(Operation::Insert, &[card.clone(), card.clone()]);

    event.emit(app_handle)?;

    Ok(card)
}
