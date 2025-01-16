use crate::database::query::{insert, upsert, upsert_or_delete};
use crate::events::Origin;
use crate::reconciler::{DatabaseChange, Reconciler};
use crate::types::model::{Card, YUpdate};
use crate::types::state::AppState;
use crate::types::util::{BytesBase64URL, UUIDv7Base64URL};
use crate::utils::{get_rw_state, get_state};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn upsert_card<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    card: Card,
    y_updates: Vec<BytesBase64URL>,
) -> eyre::Result<()> {
    let lock = get_rw_state::<R, AppState>(&app_handle)?;
    let app_state = lock.read().await;
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;
    let user_id = app_state.user.as_ref().map(|u| u.id);
    let rowids = upsert_card_impl(&app_handle, pot_id, user_id, &card, y_updates).await?;

    let reconciler = get_state::<R, Reconciler>(&app_handle)?;
    reconciler
        .send(DatabaseChange::new(rowids, Origin::local(window.label())))
        .await?;

    eyre::Ok(())
}

async fn upsert_card_impl<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    user_id: Option<UUIDv7Base64URL>,
    card: &Card,
    y_updates: Vec<BytesBase64URL>,
) -> eyre::Result<Vec<i64>> {
    let y_updates = y_updates
        .into_iter()
        .map(|data| YUpdate::new(card.id, data))
        .collect::<Vec<YUpdate>>();

    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let mut tx = pool.begin().await?;

    let mut rowids: Vec<i64> = vec![];

    insert::from_local::y_doc(
        &mut *tx,
        "outline",
        card.id,
        pot_id,
        user_id,
        card.created_at,
    )
    .await?;
    rowids
        .extend(insert::from_local::y_updates(&mut *tx, &y_updates, None, card.updated_at).await?);
    rowids.push(upsert::card(&mut *tx, card).await?);
    upsert_or_delete::card_links(&mut tx, card.id, &card.links).await?;
    upsert_or_delete::quote(&mut tx, card.id, &card.quote).await?;

    tx.commit().await?;

    Ok(rowids)
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn upsert_card<R: Runtime>(
        app_handle: &AppHandle<R>,
        pot_id: UUIDv7Base64URL,
        card: &Card,
        y_updates: Vec<BytesBase64URL>,
    ) -> eyre::Result<Vec<i64>> {
        upsert_card_impl(app_handle, pot_id, None, card, y_updates).await
    }
}
