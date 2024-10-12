use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::types::model::{Card, RawCard};

pub async fn fetch_cards_by_created_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<Card>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();
    sqlx::query_as!(
        RawCard,
        r#"
            SELECT
                cards.id, cards.outline_id, cards.fractional_index, cards.text,
                cards.version_id AS version_id,
                quotes.quoted_card_id AS quoted_card_id,
                quotes.version_id AS quote_version_id
            FROM cards
            LEFT JOIN quotes ON cards.id = quotes.card_id
            WHERE ? <= created_at AND created_at < ? AND is_deleted = false;
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await
    .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
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
        RawCard,
        r#"
            SELECT
                cards.id, cards.outline_id, cards.fractional_index, cards.text,
                cards.version_id AS version_id,
                quotes.quoted_card_id AS quoted_card_id,
                quotes.version_id AS quote_version_id
            FROM cards
            LEFT JOIN quotes ON cards.id = quotes.card_id
            WHERE ? <= cards.created_at AND cards.updated_at < ? AND is_deleted = false;
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await
    .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
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
        RawCard,
        r#"
            SELECT
                cards.id, cards.outline_id, cards.fractional_index, cards.text,
                cards.version_id AS version_id,
                quotes.quoted_card_id AS quoted_card_id,
                quotes.version_id AS quote_version_id
            FROM cards
            LEFT JOIN quotes ON cards.id = quotes.card_id
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
    .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
    .map_err(|e| anyhow!(e))
}
