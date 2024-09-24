use super::super::table::{CardsTable, CardsTableChangeEvent};
use super::super::types::{NullableBase64String, Operation::*, Origin};
use super::insert_outline;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    text: &str,
    outline_id: NullableBase64String,
    origin: Origin,
) -> anyhow::Result<CardsTable> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();
    let id = uuidv7::create().into_bytes();

    let outline_id = match outline_id.0 {
        Some(id) => id,
        None => {
            insert_outline(
                app_handle.clone(),
                None,
                NullableBase64String::none(),
                origin.clone(),
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?
            .id
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
