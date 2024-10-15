use crate::database::query;
use crate::types::model::{Outline, OutlineChangeEvent, OutlineYUpdate};
use crate::types::util::{Base64, Operation, Origin};
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn update_outline<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
    links: Vec<Base64>,
    y_updates: Vec<OutlineYUpdate>,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let mut tx = pool.begin().await?;

    query::update_outline(&mut *tx, &outline).await?;
    query::upsert_or_delete_outline_links(&mut tx, &outline.id, &links).await?;
    query::insert_outline_y_updates(&mut *tx, &outline.id, &y_updates).await?;

    tx.commit().await?;

    OutlineChangeEvent::new(Operation::Update, Origin::Local, &[outline]).emit(&app_handle)?;

    Ok(())
}
