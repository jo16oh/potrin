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

    // fetch quoted cards together
    // to avoid layout shift
    let cards: Vec<Card> = {
        let query = format!(
            r#"
                WITH c1 AS (
                    SELECT id, outline_id, fractional_index, text, quote
                    FROM cards
                    INNER JOIN card_links ON card_links.id_from = cards.id
                    WHERE (card_links.id_to IN ({}) OR cards.quote IN ({})) AND cards.is_deleted = false
                )
                SELECT * FROM c1
                UNION 
                SELECT id, outline_id, fractional_index, text, quote
                FROM cards
                WHERE id IN ((SELECT quote FROM c1)) AND is_deleted = false;

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

    // fetch quoted cards together
    // to avoid layout shift
    let cards: Vec<Card> = {
        let query = format!(
            r#"
                WITH c1 AS (
                    SELECT id, outline_id, fractional_index, text, quote 
                    FROM cards 
                    WHERE 
                        id IN ((
                            SELECT quote
                            FROM cards AS c1
                            WHERE id IN ({})
                        )) 
                        AND is_deleted = false
                )
                SELECT *
                FROM c1
                UNION 
                SELECT id, outline_id, fractional_index, text, quote 
                FROM cards 
                WHERE id IN ((SELECT quote FROM c1)) AND is_deleted = false;
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
