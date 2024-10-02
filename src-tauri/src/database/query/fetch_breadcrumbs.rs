use crate::database::types::Base64String;
use crate::database::types::NullableBase64String;
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Breadcrumb {
    pub id: Base64String,
    pub parent_id: NullableBase64String,
    pub text: Option<String>,
}

pub async fn fetch_breadcrumbs(
    parent_ids: Vec<&Base64String>,
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
            SELECT id, parent_id, text FROM breadcrumbs;
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
    use crate::database::query::create_outline;
    use crate::database::types::Origin;
    use crate::test::*;
    use tauri::{AppHandle, Manager};

    #[test]
    fn test_fetch_breadcrumbs() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        let root = create_outline(app_handle.clone(), None, Origin::Local)
            .await
            .unwrap();
        let child = create_outline(app_handle.clone(), Some(root.id), Origin::Local)
            .await
            .unwrap();
        let grand_child = create_outline(app_handle.clone(), Some(child.id), Origin::Local)
            .await
            .unwrap();

        let parent_id = grand_child.parent_id.inner().unwrap();

        let breadcrumbs = fetch_breadcrumbs(vec![parent_id], pool).await.unwrap();
        assert_eq!(breadcrumbs.len(), 2);
    }
}
