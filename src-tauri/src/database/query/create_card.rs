use crate::database::types::Base64String;

use super::super::table::{CardsTable, CardsTableChangeEvent};
use super::super::types::{Operation::*, Origin};
use super::create_outline;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn create_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    outline_id: Option<Base64String>,
    origin: Origin,
) -> anyhow::Result<CardsTable> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();
    let id = uuidv7::create().into_bytes();

    let outline_id = match outline_id {
        Some(id) => id,
        None => {
            create_outline(app_handle.clone(), None, origin.clone())
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
        "",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))?;

    CardsTableChangeEvent::new(Insert, origin, &[card.clone()]).emit(&app_handle)?;

    Ok(card)
}
