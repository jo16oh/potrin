use crate::database::query::{insert, upsert, upsert_or_delete};
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::{Paragraph, YUpdate};
use crate::types::state::AppState;
use crate::types::util::{BytesBase64URL, UUIDv7Base64URL};
use crate::utils::{get_rw_state, get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn upsert_paragraph<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    paragraph: Paragraph,
    y_updates: Vec<BytesBase64URL>,
) -> eyre::Result<()> {
    let lock = get_rw_state::<R, AppState>(&app_handle)?;
    let app_state = lock.read().await;
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;
    let user_id = app_state.user.as_ref().map(|u| u.id);
    let rowids = upsert_paragraph_impl(&app_handle, pot_id, user_id, &paragraph, y_updates).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    eyre::Ok(())
}

async fn upsert_paragraph_impl<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    user_id: Option<UUIDv7Base64URL>,
    paragraph: &Paragraph,
    y_updates: Vec<BytesBase64URL>,
) -> eyre::Result<Vec<i64>> {
    let y_updates = y_updates
        .into_iter()
        .map(|data| YUpdate::new(paragraph.id, data))
        .collect::<Vec<YUpdate>>();

    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let mut tx = pool.begin().await?;

    let mut rowids: Vec<i64> = vec![];

    insert::from_local::y_doc(&mut *tx, "outline", paragraph.id, pot_id, user_id).await?;
    rowids.extend(
        insert::from_local::y_updates(&mut *tx, &y_updates, None, paragraph.updated_at).await?,
    );
    rowids.push(upsert::paragraph(&mut *tx, paragraph).await?);
    upsert_or_delete::paragraph_links(&mut tx, paragraph.id, &paragraph.links).await?;
    upsert_or_delete::quote(&mut tx, paragraph.id, &paragraph.quote).await?;

    tx.commit().await?;

    Ok(rowids)
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn upsert_paragraph<R: Runtime>(
        app_handle: &AppHandle<R>,
        pot_id: UUIDv7Base64URL,
        paragraph: &Paragraph,
        y_updates: Vec<BytesBase64URL>,
    ) -> eyre::Result<Vec<i64>> {
        upsert_paragraph_impl(app_handle, pot_id, None, paragraph, y_updates).await
    }
}
