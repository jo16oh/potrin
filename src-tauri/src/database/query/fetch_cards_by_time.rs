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
            WITH c1 AS (
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.text,
                    cards.version_id AS version_id,
                    quotes.quoted_card_id AS quoted_card_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at AS created_at
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE ? <= created_at AND created_at < ? AND is_deleted = false
                UNION 
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.text,
                    cards.version_id AS version_id,
                    quotes.quoted_card_id AS quoted_card_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at AS created_at
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                INNER JOIN c1 ON cards.id = c1.quoted_card_id
                WHERE is_deleted = false
                ORDER BY created_at DESC
                
            )
            SELECT 
                id, outline_id, fractional_index, text, version_id, 
                quoted_card_id, quote_version_id 
            FROM c1;
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await
    .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
    .map_err(|e| anyhow!(e))
}
