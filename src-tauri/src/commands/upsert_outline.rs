use crate::database::query::{insert, upsert, upsert_or_delete};
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::{Outline, YUpdate};
use crate::types::state::AppState;
use crate::types::util::{BytesBase64, UUIDv7Base64};
use crate::utils::{get_rw_state, get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn upsert_outline<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    outline: Outline,
    y_updates: Vec<BytesBase64>,
) -> anyhow::Result<()> {
    let pot_id = window.label().try_into()?;
    let rowids = upsert_outline_impl(&app_handle, pot_id, &outline, y_updates).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    Ok(())
}

async fn upsert_outline_impl<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64,
    outline: &Outline,
    y_updates: Vec<BytesBase64>,
) -> anyhow::Result<Vec<i64>> {
    let lock = get_rw_state::<R, AppState>(app_handle)?;
    let app_state = lock.read().await;
    let user_id = app_state.user.as_ref().map(|u| u.id);

    let y_updates = y_updates
        .into_iter()
        .map(|data| YUpdate::new(outline.id, data))
        .collect::<Vec<YUpdate>>();

    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let mut tx = pool.begin().await?;

    let mut rowids: Vec<i64> = vec![];

    insert::from_local::y_doc(
        &mut *tx,
        "outline",
        outline.id,
        pot_id,
        user_id,
        outline.created_at,
    )
    .await?;
    rowids.extend(
        insert::from_local::y_updates(&mut *tx, &y_updates, None, outline.updated_at).await?,
    );
    rowids.push(upsert::outline(&mut *tx, outline).await?);
    upsert_or_delete::outline_links(&mut tx, outline.id, &outline.links).await?;

    tx.commit().await?;

    Ok(rowids)
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn upsert_outline<R: Runtime>(
        app_handle: &AppHandle<R>,
        pot_id: UUIDv7Base64,
        outline: &Outline,
        y_updates: Vec<BytesBase64>,
    ) -> anyhow::Result<Vec<i64>> {
        upsert_outline_impl(app_handle, pot_id, outline, y_updates).await
    }
}
