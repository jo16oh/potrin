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

    create_version_impl(&app_handle, pot_id, version_id).await
}

pub async fn create_version_impl<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    version_id: UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
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

    let updated_doc_ids = merged_updates
        .iter()
        .map(|u| u.y_doc_id)
        .collect::<Vec<UUIDv7Base64URL>>();

    delete::y_updates(&mut *tx, &unversioned_update_ids).await?;
    insert::from_local::version(&mut *tx, pot_id, version_id).await?;
    insert::from_local::y_updates(&mut *tx, &merged_updates).await?;
    insert::y_doc_trees_of_version(&mut *tx, &updated_doc_ids, version_id).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{database::test::create_mock_pot, run_in_mock_app};
    use tauri::test::MockRuntime;

    pub async fn create_version<R: tauri::Runtime>(
        app_handle: &AppHandle<R>,
        pot_id: UUIDv7Base64URL,
        version_id: UUIDv7Base64URL,
    ) -> eyre::Result<()> {
        create_version_impl(app_handle, pot_id, version_id).await
    }

    #[test]
    fn test_version_triggers() {
        run_in_mock_app!(test_version_triggers_impl);
    }

    async fn test_version_triggers_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pool = get_state::<MockRuntime, SqlitePool>(app_handle)?;
        let pot = create_mock_pot(app_handle).await;
        let v1 = UUIDv7Base64URL::new();
        let v2 = UUIDv7Base64URL::new();
        let v3 = UUIDv7Base64URL::new();
        create_version(app_handle, pot.id, v1).await?;
        create_version(app_handle, pot.id, v2).await?;
        create_version(app_handle, pot.id, v3).await?;

        let r = sqlx::query_scalar::<_, i64>(
            r#"
                SELECT branch_id
                FROM versions_branch_id;
            "#,
        )
        .fetch_all(pool)
        .await?;

        assert_eq!(r.len(), 3);
        for id in r {
            assert_eq!(id, 0);
        }

        let r = sqlx::query_scalar::<_, UUIDv7Base64URL>(
            r#"
                SELECT id
                FROM version_heads;
            "#,
        )
        .fetch_all(pool)
        .await?;

        assert_eq!(r.len(), 1);
        assert_eq!(v3, r[0]);

        let r = sqlx::query_scalar::<_, UUIDv7Base64URL>(
            r#"
                SELECT prev_id 
                FROM prev_versions
                WHERE id = ?;
            "#,
        )
        .bind(v3)
        .fetch_all(pool)
        .await?;

        assert_eq!(r.len(), 1);
        assert_eq!(v2, r[0]);

        let r = sqlx::query_scalar::<_, UUIDv7Base64URL>(
            r#"
                SELECT prev_id 
                FROM prev_versions
                WHERE id = ?;
            "#,
        )
        .bind(v2)
        .fetch_all(pool)
        .await?;

        assert_eq!(r.len(), 1);
        assert_eq!(v1, r[0]);

        let r = sqlx::query_scalar::<_, UUIDv7Base64URL>(
            r#"
                SELECT prev_id 
                FROM prev_versions
                WHERE id = ?;
            "#,
        )
        .bind(v1)
        .fetch_all(pool)
        .await?;

        assert_eq!(r.len(), 0);

        Ok(())
    }
}
