use crate::types::model::Card;
use crate::types::model::Outline;
use crate::types::model::RawCard;
use crate::types::util::Base64;
use sqlx::SqlitePool;

pub async fn fetch_relation_back(
    pool: &SqlitePool,
    outline_ids: &[Base64],
    card_ids: &[Base64],
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT id, parent_id, fractional_index, text
                FROM outline_links
                INNER JOIN outlines ON outline_links.id_from = outlines.id
                WHERE outlines.is_deleted = false AND id_to IN ({});
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    let cards: Vec<Card> = {
        let query = format!(
            r#"
                WITH c1 AS (
                    SELECT 
                        cards.id, cards.outline_id, cards.fractional_index, cards.text,
                        cards.version_id AS version_id,
                        quotes.quoted_card_id AS quoted_card_id,
                        quotes.version_id AS quote_version_id
                    FROM card_links
                    INNER JOIN cards ON card_links.id_from = cards.id
                    LEFT JOIN quotes ON quotes.card_id = cards.id
                    WHERE card_links.id_to IN ({}) AND cards.is_deleted = false
                    UNION 
                    SELECT 
                        cards.id, cards.outline_id, cards.fractional_index, cards.text,
                        cards.version_id AS version_id,
                        quotes.quoted_card_id AS quoted_card_id,
                        quotes.version_id AS quote_version_id
                    FROM quotes
                    INNER JOIN cards ON quotes.card_id = cards.id
                    WHERE quotes.quoted_card_id IN ({}) AND cards.is_deleted = false
                    UNION
                    SELECT 
                        cards.id, cards.outline_id, cards.fractional_index, cards.text,
                        cards.version_id AS version_id,
                        quotes.quoted_card_id AS quoted_card_id,
                        quotes.version_id AS quote_version_id
                    FROM cards
                    INNER JOIN c1 ON c1.quoted_card_id = cards.id
                    INNER JOIN quotes ON quotes.card_id = cards.id
                    WHERE cards.is_deleted = false
                )
                SELECT * FROM c1;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
            card_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
        );

        let mut query_builder = sqlx::query_as::<_, RawCard>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        for id in card_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .fetch_all(pool)
            .await
            .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())?
    };

    Ok((outlines, cards))
}

pub async fn fetch_relation_forward(
    pool: &SqlitePool,
    outline_ids: &[Base64],
    card_ids: &[Base64],
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT id, parent_id, fractional_index, text
                FROM outlines
                INNER JOIN outline_links ON outline_links.id_to = outlines.id
                INNER JOIN card_links ON card_links.id_to = outlines.id
                WHERE outline_links.id_from IN ({}) AND outlines.is_deleted = false;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    let cards: Vec<Card> = {
        let query = format!(
            r#"
                WITH c1 AS (
                    SELECT 
                        cards.id, cards.outline_id, cards.fractional_index, cards.text,
                        cards.version_id AS version_id,
                        q2.quoted_card_id AS quoted_card_id,
                        q2.version_id AS quote_version_id
                    FROM cards
                    INNER JOIN quotes AS q1 ON q1.quoted_card_id = cards.id
                    LEFT JOIN quotes AS q2 ON q2.card_id = cards.id
                    WHERE q1.card_id IN ({}) AND cards.is_deleted = false
                    UNION
                    SELECT 
                        cards.id, cards.outline_id, cards.fractional_index, cards.text,
                        cards.version_id AS version_id,
                        quotes.quoted_card_id AS quoted_card_id,
                        quotes.version_id AS quote_version_id
                    FROM cards
                    INNER JOIN c1 ON c1.quoted_card_id = cards.id
                    LEFT JOIN quotes ON quotes.card_id = cards.id
                    WHERE cards.is_deleted = false
                )
                SELECT * FROM c1;
            "#,
            card_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, RawCard>(&query);

        for id in card_ids {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .fetch_all(pool)
            .await
            .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())?
    };

    Ok((outlines, cards))
}
