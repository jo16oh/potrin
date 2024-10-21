use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow)]
struct QueryResult {
    id: Base64,
}

pub async fn fetch_card_ids_by_outline_id(
    pool: &SqlitePool,
    outline_ids: Vec<&Base64>,
) -> anyhow::Result<Vec<Base64>> {
    let query = format!(
        r#"
            SELECT id
            FROM cards
            WHERE outline_ids IN ({});
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, QueryResult>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|r| r.into_iter().map(|r| r.id).collect::<Vec<Base64>>())
        .map_err(|e| anyhow!(e))
}
