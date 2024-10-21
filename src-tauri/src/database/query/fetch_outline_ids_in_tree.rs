use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqlitePool;

struct QueryResult {
    pub id: Base64,
}

pub async fn fetch_outline_ids_in_tree(
    pool: &SqlitePool,
    outline_id: &Base64,
) -> anyhow::Result<Vec<Base64>> {
    sqlx::query_as!(
        QueryResult,
        r#"
            WITH RECURSIVE tree AS (
                SELECT id, parent_id 
                FROM outlines
                WHERE id = ?
                UNION ALL
                SELECT outlines.id, outlines.parent_id
                FROM outlines
                INNER JOIN tree ON outlines.id = tree.parent_id
                UNION ALL
                SELECT outlines.id, outlines.parent_id
                FROM outlines
                INNER JOIN tree ON outlines.parent_id = tree.id
            )
            SELECT DISTINCT id
            FROM tree;
        "#,
        outline_id,
    )
    .fetch_all(pool)
    .await
    .map(|r| r.into_iter().map(|r| r.id).collect::<Vec<Base64>>())
    .map_err(|e| anyhow!(e))
}
