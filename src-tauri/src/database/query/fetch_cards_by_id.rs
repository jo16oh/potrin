use crate::types::{model::Card, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_cards_by_id(
    pool: &SqlitePool,
    outline_ids: &[&Base64],
) -> anyhow::Result<Vec<Card>> {
    let query = format!(
        r#"
            SELECT * FROM cards WHERE outline_id IN ({}) AND is_deleted = false;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = query_as::<_, Card>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
