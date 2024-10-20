use crate::types::{model::CardYUpdate, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_card_y_updates_by_card_id(
    pool: &SqlitePool,
    card_id: &Base64,
) -> anyhow::Result<Vec<CardYUpdate>> {
    query_as!(
        CardYUpdate,
        r#"
            SELECT id, data
            FROM card_y_updates
            WHERE card_id = ?;
        "#,
        card_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e))
}
