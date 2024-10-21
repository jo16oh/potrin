use crate::types::model::Outline;
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqlitePool;

pub async fn fetch_outline_tree(
    id: &Base64,
    depth: Option<u32>,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<Outline>> {
    match depth {
        Some(depth) => {
            sqlx::query_as!(
                Outline,
                r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, parent_id, fractional_index, text, 0 AS depth
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.parent_id, child.fractional_index, child.text,
                        parent.depth + 1 AS depth
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false AND depth <= ?
                )
                SELECT
                    id, parent_id, fractional_index, text
                FROM outline_tree;
                "#,
                id,
                depth
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Outline,
                r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, parent_id, fractional_index, text
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.parent_id, child.fractional_index, child.text
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false
                )
                SELECT * FROM outline_tree;
                "#,
                id
            )
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| anyhow!(e.to_string()))
}
