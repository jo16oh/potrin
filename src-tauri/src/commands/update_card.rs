use crate::database::query;
use crate::types::model::{Card, CardChangeEvent, CardYUpdate};
use crate::types::util::{Base64, Operation, Origin};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn update_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
    links: Vec<Base64>,
    y_updates: Vec<CardYUpdate>,
) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    query::update_card(&mut *tx, &card).await?;
    query::upsert_or_delete_card_links(&mut tx, &card.id, &links).await?;
    query::insert_card_y_updates(&mut *tx, &card.id, &y_updates).await?;

    if let Some(ref quote) = card.quote {
        query::upsert_quote(&mut *tx, &card.id, quote).await?;
    } else {
        query::delete_quote(&mut *tx, &card.id).await?;
    }

    tx.commit().await?;

    CardChangeEvent::new(Operation::Update, Origin::Local, &[card]).emit(&app_handle)?;

    Ok(())
}
