use crate::types::model::LinkCount;
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqlitePool;

pub async fn count_relation(
    pool: &SqlitePool,
    outline_ids: &[Base64],
    card_ids: &[Base64],
) -> anyhow::Result<Vec<LinkCount>> {
    let query = format!(
        r#"
            SELECT
                id,
                (
                    (
                        SELECT COUNT(*)
                        FROM outline_links
                        WHERE outline_links.id_to = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM card_links
                        WHERE card_links.id_to = this.id
                    )
                ) AS back,
                (
                    SELECT COUNT(*)
                    FROM outline_links
                    WHERE outline_links.id_from = this.id
                ) AS forward
            FROM outlines AS this
            WHERE id IN ({})
            UNION ALL
            SELECT
                id,
                (
                    SELECT COUNT(*)
                    FROM quotes
                    WHERE quoted_card_id = this.id
                ) AS back,
                (
                    (
                        SELECT COUNT(*)
                        FROM card_links
                        WHERE card_links.id_from = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE card_id = this.id
                    )
                ) AS forward
            FROM cards AS this
            WHERE id IN ({});
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

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in card_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}

pub async fn count_relation_recursively(
    pool: &SqlitePool,
    outline_ids: &[Base64],
    card_ids: &[Base64],
) -> anyhow::Result<Vec<LinkCount>> {
    let query = format!(
        r#"
                WITH RECURSIVE tree AS (
                    SELECT id, id AS root_id
                    FROM outlines
                    WHERE id IN ({}) AND is_deleted = false
                    UNION ALL
                    SELECT child.id, parent.root_id AS root_id
                    FROM tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false
                ),
                tree_cards AS (
                    SELECT cards.id, tree.root_id
                    FROM cards
                    INNER JOIN tree ON cards.outline_id = tree.id
                )
                SELECT
                    id,
                    (
                        (
                            SELECT COUNT(*)
                            FROM outline_links
                            WHERE outline_links.id_to IN ((
                                SELECT id
                                FROM tree
                                WHERE tree.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_to IN ((
                                SELECT id
                                FROM tree
                                WHERE tree.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM quotes
                            WHERE quoted_card_id
                             IN ((
                                SELECT id
                                FROM tree_cards
                                WHERE tree_cards.root_id = this.id
                            ))
                        )
                    ) AS back,
                    (
                        SELECT COUNT(*)
                        FROM outline_links
                        WHERE outline_links.id_from IN ((
                            SELECT id
                            FROM tree
                            WHERE tree.root_id = this.id
                        ))
                    )
                    +
                    (
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_from IN ((
                                SELECT id
                                FROM tree_cards
                                WHERE tree_cards.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM quotes
                            WHERE 
                                card_id IN ((
                                    SELECT id
                                    FROM tree_cards
                                    WHERE tree_cards.root_id = this.id
                                )) 
                        )
                    ) AS forward
                FROM tree AS this
                WHERE id = root_id
                UNION ALL
                SELECT DISTINCT
                    id,
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE quoted_card_id = this.id
                    ) AS back,
                    (
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_from = this.id
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM quotes
                            WHERE card_id = this.id
                        )
                    ) AS forward
                FROM tree_cards AS this
                WHERE id IN ({});
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

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in card_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
