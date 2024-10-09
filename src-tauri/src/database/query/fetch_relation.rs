use crate::types::model::Card;
use crate::types::model::Outline;
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
                SELECT
                    id, parent_id, fractional_index, text
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
                SELECT
                    id, outline_id, fractional_index, text
                FROM cards
                INNER JOIN card_links ON card_links.id_from = cards.id
                INNER JOIN card_quotes ON card_links.id_from = cards.id
                WHERE (card_links.id_to IN ({}) OR card_quotes.id_to IN ({})) AND cards.is_deleted = false;
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
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Card>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}

pub async fn fetch_relation_forward(
    pool: &SqlitePool,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    id, parent_id, fractional_index, text
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
                SELECT
                    id, outline_id, fractional_index, text
                FROM card_quotes
                INNER JOIN cards ON card_quotes.id_to = cards.id
                WHERE cards.is_deleted = false AND card_quotes.id_from IN ({});
            "#,
            card_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Card>(&query);

        for id in card_ids {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}
