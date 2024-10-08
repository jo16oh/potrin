use crate::{
    database::{
        query::insert_outline_y_updates,
        table::{Outline, OutlineChangeEvent, OutlineYUpdate},
        types::{Base64, Operation::*, Origin},
    },
    state::types::AppState,
};
use anyhow::anyhow;
use sqlx::SqlitePool;
use std::sync::RwLock;
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
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut tx = pool.begin().await?;

    let pot_id = {
        let lock = app_handle
            .try_state::<RwLock<AppState>>()
            .ok_or(anyhow!("failed to get app state"))?
            .inner();

        let app_state = lock.read().map_err(|e| anyhow!(e.to_string()))?;

        let id = app_state
            .pot
            .as_ref()
            .ok_or(anyhow!("failed to get pot state"))?
            .id
            .clone();

        Base64::from(id)
    };

    let outline: Outline = sqlx::query_as!(
        Outline,
        r#"
            INSERT INTO outlines (id, pot_id, parent_id, fractional_index, text)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, parent_id, fractional_index, text;"#,
        outline.id,
        pot_id,
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
