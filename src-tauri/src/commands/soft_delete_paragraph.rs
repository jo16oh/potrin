use crate::database::query::delete;
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::Paragraph;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn soft_delete_paragraph<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    paragraph: Paragraph,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let rowids = delete::soft::paragraphs(pool, &[paragraph.id]).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    eyre::Ok(())
}
