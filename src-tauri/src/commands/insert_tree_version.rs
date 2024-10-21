use std::collections::HashMap;

use crate::database::query::{
    delete_card_y_updates, delete_outline_y_updates, fetch_card_ids_by_outline_id,
    fetch_outline_ids_in_tree, fetch_unversioned_card_y_updates,
    fetch_unversioned_outline_y_updates, insert_card_y_updates_many, insert_outline_y_updates_many,
    insert_version,
};
use crate::types::state::AppState;
use crate::types::util::Base64;
use crate::utils::get_state;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert_tree_version<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    outline_id: Base64,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let app_state = get_state::<R, AppState>(&app_handle)?;
    let pot_id = &app_state
        .pot
        .as_ref()
        .ok_or(anyhow!("pot state is not set"))?
        .id;

    let outline_ids = fetch_outline_ids_in_tree(pool, &outline_id).await?;
    let card_ids = fetch_card_ids_by_outline_id(pool, outline_ids.iter().collect()).await?;

    let unversioned_outline_y_updates =
        fetch_unversioned_outline_y_updates(pool, outline_ids.iter().collect()).await?;
    let unversioned_card_y_updates =
        fetch_unversioned_card_y_updates(pool, card_ids.iter().collect()).await?;

    let merged_outline_y_updates = {
        let mut updates_map: HashMap<&Base64, Vec<&Vec<u8>>> = HashMap::new();

        for update in unversioned_outline_y_updates.iter() {
            #[allow(clippy::unwrap_or_default)]
            updates_map
                .entry(&update.outline_id)
                .or_insert_with(Vec::new)
                .push(&update.data);
        }

        updates_map
            .into_iter()
            .map(|(outline_id, data)| {
                yrs::merge_updates_v2(data)
                    .map_err(|e| anyhow!(e))
                    .map(|data| (outline_id, data))
            })
            .collect::<anyhow::Result<Vec<(&Base64, Vec<u8>)>>>()?
    };

    let merged_card_y_updates = {
        let mut updates_map: HashMap<&Base64, Vec<&Vec<u8>>> = HashMap::new();

        for update in unversioned_card_y_updates.iter() {
            #[allow(clippy::unwrap_or_default)]
            updates_map
                .entry(&update.card_id)
                .or_insert_with(Vec::new)
                .push(&update.data);
        }

        updates_map
            .into_iter()
            .map(|(card_id, data)| {
                yrs::merge_updates_v2(data)
                    .map_err(|e| anyhow!(e))
                    .map(|data| (card_id, data))
            })
            .collect::<anyhow::Result<Vec<(&Base64, Vec<u8>)>>>()?
    };

    let mut tx = pool.begin().await?;

    delete_outline_y_updates(
        &mut *tx,
        unversioned_outline_y_updates
            .iter()
            .map(|u| &u.id)
            .collect(),
    )
    .await?;
    delete_card_y_updates(
        &mut *tx,
        unversioned_card_y_updates.iter().map(|u| &u.id).collect(),
    )
    .await?;
    insert_outline_y_updates_many(&mut *tx, merged_outline_y_updates).await?;
    insert_card_y_updates_many(&mut *tx, merged_card_y_updates).await?;
    insert_version(
        &mut tx,
        pot_id,
        outline_ids.iter().collect(),
        card_ids.iter().collect(),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}
