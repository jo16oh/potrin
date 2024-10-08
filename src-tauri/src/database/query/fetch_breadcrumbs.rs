use crate::database::types::Base64;
use crate::database::types::NullableBase64;
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Breadcrumb {
    pub id: Base64,
    pub parent_id: NullableBase64,
    pub text: Option<String>,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_breadcrumbs(
    parent_ids: Vec<&Base64>,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<Breadcrumb>> {
    let query = format!(
        r#"
            WITH RECURSIVE breadcrumbs AS (
                SELECT
                    id, parent_id, text
                FROM outlines
                WHERE id IN ({}) AND is_deleted = false
                UNION ALL
                SELECT
                    parent.id, parent.parent_id, parent.text
                FROM breadcrumbs AS child
                JOIN outlines AS parent ON parent.id = child.parent_id
                WHERE parent.is_deleted = false
            )
            SELECT DISTINCT id, parent_id, text FROM breadcrumbs;
        "#,
        parent_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = query_as::<_, Breadcrumb>(&query);

    for id in parent_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::table::OutlineYUpdate;
    use crate::database::{query::insert_outline, table::Outline};
    use crate::test::*;
    use tauri::{AppHandle, Manager};

    #[test]
    fn test_fetch_breadcrumbs() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        let root = insert_outline(
            app_handle.clone(),
            Outline::new(None),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        let child = insert_outline(
            app_handle.clone(),
            Outline::new(Some(&root.id)),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        let grand_child = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::from(child.id.clone()),
            fractional_index: String::new(),
            text: Some(String::new()),
        };

        insert_outline(
            app_handle.clone(),
            grand_child.clone(),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        let parent_id = grand_child.parent_id.inner().unwrap();

        let breadcrumbs = fetch_breadcrumbs(vec![parent_id], pool).await.unwrap();
        assert_eq!(breadcrumbs.len(), 2);
    }
}
