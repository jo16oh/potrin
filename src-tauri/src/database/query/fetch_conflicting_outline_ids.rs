use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow)]
struct QueryResult {
    id: Base64,
}

pub async fn fetch_conflictint_outline_ids(
    pool: &SqlitePool,
    outline_ids: Vec<&Base64>,
) -> anyhow::Result<Vec<Base64>> {
    let query = format!(
        r#"
            SELECT o1.id 
            FROM outlines AS o1
            WHERE 
                id IN ({})
                AND EXISTS (
                    SELECT 1 
                    FROM outlines AS o2
                    WHERE o1.text = o2.text
                );
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
        .map(|vec| vec.into_iter().map(|r| r.id).collect())
        .map_err(|e| anyhow!(e))
}
