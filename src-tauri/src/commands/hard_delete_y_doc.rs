use crate::database::query::delete;
use crate::events::Origin;
use crate::reconciler::DatabaseChange;
use crate::reconciler::Reconciler;
use crate::types::model::{Card, Outline};
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn hard_delete_outline<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    outline: Outline,
) -> eyre::Result<()> {
    let rowid = hard_delete_y_doc(&app_handle, outline.id).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(
            vec![rowid],
            Origin::local(window.label()),
        ))
        .await?;
    eyre::Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn hard_delete_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    card: Card,
) -> eyre::Result<()> {
    let rowid = hard_delete_y_doc(&app_handle, card.id).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(
            vec![rowid],
            Origin::local(window.label()),
        ))
        .await?;
    eyre::Ok(())
}

async fn hard_delete_y_doc<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    y_doc_id: UUIDv7Base64URL,
) -> eyre::Result<i64> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    delete::y_doc(pool, y_doc_id).await
}
