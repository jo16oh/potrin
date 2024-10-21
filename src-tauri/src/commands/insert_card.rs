use crate::database::query;
use crate::types::model::{Card, CardChangeEvent, YUpdate};
use crate::types::util::{Operation, Origin};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
    y_updates: Vec<YUpdate>,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let mut tx = pool.begin().await?;

    query::insert_card(&mut *tx, &card).await?;
    query::insert_card_y_updates(&mut *tx, &card.id, &y_updates).await?;

    if let Some(ref quote) = card.quote {
        query::upsert_quote(&mut *tx, &card.id, quote).await?;
    }

    tx.commit().await?;

    CardChangeEvent::new(Operation::Insert, Origin::Local, &[card]).emit(&app_handle)?;

    Ok(())
}
