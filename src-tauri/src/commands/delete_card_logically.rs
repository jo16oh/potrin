use crate::database::query;
use crate::types::model::{Card, CardChangeEvent};
use crate::types::util::{Operation, Origin};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn delete_card_logically<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    card: Card,
) -> anyhow::Result<()> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    query::delete_outline_logically(pool, &card.id).await?;

    CardChangeEvent::new(Operation::Delete, Origin::Local, &[card]).emit(&app_handle)?;

    Ok(())
}
