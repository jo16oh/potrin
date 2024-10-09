use crate::database::query;
use crate::types::model::{Card, CardChangeEvent, CardYUpdate};
use crate::types::util::{Operation, Origin};
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
) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    query::insert_card(&mut *tx, &card).await?;
    query::insert_card_y_updates(&mut *tx, &card.id, &y_updates).await?;

    tx.commit().await?;

    CardChangeEvent::new(Operation::Insert, Origin::Local, &[card]).emit(&app_handle)?;

    Ok(())
}
