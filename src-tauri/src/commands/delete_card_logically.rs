use crate::database::query;
use crate::types::model::{Card, CardChangeEvent};
use crate::types::util::{Operation, Origin};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn delete_card_logically<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    query::delete_card_logically(pool, &card.id).await?;

    CardChangeEvent::new(Operation::Delete, Origin::Local, &[card]).emit(&app_handle)?;

    Ok(())
}
