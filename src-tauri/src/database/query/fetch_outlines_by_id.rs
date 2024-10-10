use crate::types::{model::Outline, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_outlines_by_id(
    pool: &SqlitePool,
    outline_ids: &[&Base64],
) -> anyhow::Result<Vec<Outline>> {
    let query = format!(
        r#"
            SELECT id, parent_id, fractional_index, text
            FROM outlines
            WHERE id IN ({}) AND is_deleted = false;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = query_as::<_, Outline>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
