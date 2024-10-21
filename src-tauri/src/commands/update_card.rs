use crate::database::query;
use crate::types::model::{Card, CardChangeEvent, YUpdate};
use crate::types::util::{Base64, Operation, Origin};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn update_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
    links: Vec<Base64>,
    y_updates: Vec<YUpdate>,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

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
