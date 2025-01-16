use crate::database::query::delete;
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::Card;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn soft_delete_card<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    card: Card,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let rowids = delete::soft::cards(pool, &[card.id]).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    eyre::Ok(())
}
