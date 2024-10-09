use crate::types::util::Base64;
use sqlx::prelude::FromRow;
use sqlx::SqlitePool;

#[derive(FromRow)]
struct QueryResult {
    id: Base64,
}

pub async fn fetch_descendant_ids(
    pool: &SqlitePool,
    outline_ids: &[Base64],
    include_cards: bool,
) -> anyhow::Result<(Vec<Base64>, Vec<Base64>)> {
    let outline_ids = {
        let query = format!(
            r#"
                    WITH RECURSIVE outline_tree AS (
                        SELECT id
                        FROM outlines
                        WHERE id = {} AND is_deleted = false
                        UNION ALL
                        SELECT child.id
                        FROM outline_tree AS parent
                        JOIN outlines AS child ON parent.id = child.parent_id
                        WHERE child.is_deleted = false
                    )
                    SELECT id FROM outline_tree;
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
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<Base64>>()
    };

    let card_ids = if include_cards {
        let query = format!(
            r#"
                SELECT id FROM cards WHERE outline_id IN ({}) AND is_deleted = false;
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
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<Base64>>()
    } else {
        vec![]
    };

    Ok((outline_ids, card_ids))
}
