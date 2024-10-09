use crate::database::{
    query::insert_card_y_updates,
    table::{Card, CardChangeEvent, CardYUpdate},
    types::{Operation::*, Origin},
};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
    y_updates: Vec<CardYUpdate>,
) -> anyhow::Result<Card> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    let card = sqlx::query_as!(
        Card,
        r#"
            INSERT INTO cards (id, outline_id, fractional_index, text)
            VALUES (?, ?, ?, ?)
            RETURNING id, outline_id, fractional_index, text;
        "#,
        card.id,
        card.outline_id,
        card.fractional_index,
        card.text,
    )
    .fetch_one(&mut *tx)
    .await?;

    insert_card_y_updates(&mut tx, &card.id, y_updates).await?;

    tx.commit().await?;

    CardChangeEvent::new(Insert, Origin::Local, &[card.clone()]).emit(&app_handle)?;

    Ok(card)
}
