use crate::{
    database::{
        query::insert_outline_y_updates,
        table::{Outline, OutlineChangeEvent, OutlineYUpdate},
        types::{Operation::*, Origin},
    },
    state::get_app_state,
};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    outline: Outline,
    y_updates: Vec<OutlineYUpdate>,
) -> anyhow::Result<Outline> {
    let pot = get_app_state(app_handle.clone())
        .map_err(|e| anyhow!(e))?
        .pot
        .ok_or(anyhow!("pot state is not set"))?;

    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    let outline: Outline = sqlx::query_as!(
        Outline,
        r#"
            INSERT INTO outlines (id, pot_id, parent_id, fractional_index, text)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, parent_id, fractional_index, text;"#,
        outline.id,
        pot.id,
        outline.parent_id,
        outline.fractional_index,
        outline.text
    )
    .fetch_one(&mut *tx)
    .await?;

    insert_outline_y_updates(&mut tx, &outline.id, y_updates).await?;

    tx.commit().await?;

    OutlineChangeEvent::new(Insert, Origin::Local, &[outline.clone()]).emit(&app_handle)?;

    Ok(outline)
}
