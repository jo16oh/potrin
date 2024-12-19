use crate::database::query::*;
use crate::types::model::YUpdate;
use crate::types::util::{BytesBase64, UUIDv7Base64};
use crate::utils::get_state;
use anyhow::Context;
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn create_version<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    version_id: UUIDv7Base64,
) -> anyhow::Result<()> {
    let pot_id: UUIDv7Base64 = window.label().try_into()?;

    create_version_inner(app_handle, pot_id, version_id).await
}

async fn create_version_inner<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    pot_id: UUIDv7Base64,
    version_id: UUIDv7Base64,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let mut updates_map: HashMap<UUIDv7Base64, Vec<BytesBase64>> = HashMap::new();
    let mut unversioned_update_ids: Vec<UUIDv7Base64> = vec![];

    for update in fetch::unversioned_y_updates(pool).await? {
        unversioned_update_ids.push(update.id);
        #[allow(clippy::unwrap_or_default)]
        updates_map
            .entry(update.y_doc_id)
            .or_insert_with(Vec::new)
            .push(update.data);
    }

    let merged_updates = updates_map
        .into_iter()
        .map(|(y_doc_id, data)| {
            yrs::merge_updates_v2(
                data.into_iter()
                    .map(|d| d.to_vec())
                    .collect::<Vec<Vec<u8>>>(),
            )
            .map(|data| YUpdate::new(y_doc_id, BytesBase64::from(data)))
            .context("failed to merge updates")
        })
        .collect::<anyhow::Result<Vec<YUpdate>>>()?;

    let now = Utc::now().timestamp_millis();

    let mut tx = pool.begin().await?;

    delete::y_updates(&mut *tx, &unversioned_update_ids).await?;
    insert::from_local::version(&mut *tx, pot_id, version_id, now).await?;
    insert::from_local::y_updates(&mut *tx, &merged_updates, Some(version_id), now).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn create_version<R: tauri::Runtime>(
        app_handle: AppHandle<R>,
        pot_id: UUIDv7Base64,
        version_id: UUIDv7Base64,
    ) -> anyhow::Result<()> {
        create_version_inner(app_handle, pot_id, version_id).await
    }
}
