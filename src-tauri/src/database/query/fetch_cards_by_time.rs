use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::types::model::Card;

pub async fn fetch_cards_by_created_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<Card>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();
    sqlx::query_as!(
        Card,
        r#"
            SELECT id, outline_id, fractional_index, text, quote
            FROM cards
            WHERE ? <= created_at AND created_at < ? AND is_deleted = false;
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e))
}

pub async fn fetch_cards_by_updated_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<Card>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();
    sqlx::query_as!(
        Card,
        r#"
            SELECT id, outline_id, fractional_index, text, quote
            FROM cards
            WHERE ? <= updated_at AND updated_at < ? AND is_deleted = false;
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e))
}

pub async fn fetch_cards_by_created_at_and_updated_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<Card>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();
    sqlx::query_as!(
        Card,
        r#"
            SELECT id, outline_id, fractional_index, text, quote
            FROM cards
            WHERE
                ((? <= updated_at AND updated_at < ?) OR (? <= created_at AND created_at < ?))
                AND is_deleted = false;
        "#,
        from,
        to,
        from,
        to
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e))
}
