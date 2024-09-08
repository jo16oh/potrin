use super::super::table::{types::Operation::*, types::Origin, CardsTable, CardsTableChangeEvent};
use super::super::POOL;
use super::insert_outline;
use crate::types::NullableBase64String;
use crate::utils::get_once_lock;
use anyhow::anyhow;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    text: &str,
    outline_id: Option<Vec<u8>>,
    origin: Origin,
) -> anyhow::Result<CardsTable> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().into_bytes();

    let outline_id = match outline_id {
        Some(id) => id,
        None => {
            let outline = insert_outline(
                app_handle.clone(),
                None,
                NullableBase64String::none(),
                origin.clone(),
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
            outline.id.to_bytes()?
        }
    };

    let card = sqlx::query_as!(
        CardsTable,
        r#"
            INSERT INTO cards (id, outline_id, fractional_index, text)
            VALUES (?, ?, ?, ?)
            RETURNING *;
        "#,
        id,
        outline_id,
        "",
        text,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))?;

    CardsTableChangeEvent::new(Insert, origin, &[card.clone()]).emit(&app_handle)?;

    Ok(card)
}
