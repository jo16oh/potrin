use crate::database::query::*;
use crate::types::model::YUpdate;
use crate::types::util::{BytesBase64URL, UUIDv7Base64URL};
use crate::utils::get_state;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tauri::{AppHandle, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn create_version<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    version_id: UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;

    create_version_impl(app_handle, pot_id, version_id).await
}

async fn create_version_impl<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    version_id: UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let mut tx = pool.begin().await?;

    let mut updates_map: HashMap<UUIDv7Base64URL, Vec<(BytesBase64URL, i64)>> = HashMap::new();
    let mut unversioned_update_ids: Vec<UUIDv7Base64URL> = vec![];

    for update in fetch::unversioned_y_updates(pool).await? {
        unversioned_update_ids.push(update.id);
        #[allow(clippy::unwrap_or_default)]
        updates_map
            .entry(update.y_doc_id)
            .or_insert_with(Vec::new)
            .push((update.data, update.timestamp));
    }

    let merged_updates = updates_map
        .into_iter()
        .map(|(y_doc_id, updates)| {
            let (y_updates, timestamps): (Vec<BytesBase64URL>, Vec<i64>) =
                updates.into_iter().unzip();

            let merged_update = yrs::merge_updates_v2(y_updates)?;

            let timestamp = timestamps.into_iter().max().unwrap();

            Ok(YUpdate::new(
                y_doc_id,
                merged_update.into(),
                Some(version_id),
                timestamp,
            ))
        })
        .collect::<eyre::Result<Vec<YUpdate>>>()?;

    delete::y_updates(&mut *tx, &unversioned_update_ids).await?;
    insert::from_local::version(&mut *tx, pot_id, version_id).await?;
    insert::from_local::y_updates(&mut *tx, &merged_updates).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn create_version<R: tauri::Runtime>(
        app_handle: AppHandle<R>,
        pot_id: UUIDv7Base64URL,
        version_id: UUIDv7Base64URL,
    ) -> eyre::Result<()> {
        create_version_impl(app_handle, pot_id, version_id).await
    }
}
