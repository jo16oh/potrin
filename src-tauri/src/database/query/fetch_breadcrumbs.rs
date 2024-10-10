use crate::types::{model::Breadcrumb, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_breadcrumbs(
    pool: &SqlitePool,
    parent_ids: Vec<Base64>,
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

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
