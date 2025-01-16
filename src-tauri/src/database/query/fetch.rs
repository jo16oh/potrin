use crate::{
    search_engine::DeleteTarget,
    types::{
        model::{
            Ancestor, Card, CardForIndex, LinkCount, Oplog, Outline, OutlineForIndex, Path, Pot,
            RawCard, RawCardForIndex, YUpdate,
        },
        util::{BytesBase64URL, UUIDv7Base64URL},
    },
};
use chrono::{DateTime, Utc};
use eyre::Result;
use sqlx::{prelude::FromRow, SqlitePool};

pub async fn pots(pool: &SqlitePool) -> Result<Vec<Pot>> {
    sqlx::query_as::<_, Pot>("SELECT * FROM pots;")
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn path(pool: &SqlitePool, outline_id: UUIDv7Base64URL) -> Result<Path> {
    let query = r#"
        SELECT path
        FROM outline_paths
        WHERE id = ?;
    "#;

    let mut query_builder = sqlx::query_scalar::<_, Path>(query);

    query_builder = query_builder.bind(outline_id);

    query_builder
        .fetch_one(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn ancestors(pool: &SqlitePool, parent_ids: &[UUIDv7Base64URL]) -> Result<Vec<Ancestor>> {
    let query = format!(
        r#"
            WITH RECURSIVE ancestors AS (
                SELECT
                    id, parent_id, text
                FROM outlines
                WHERE id IN ({})
                UNION ALL
                SELECT
                    parent.id, parent.parent_id, parent.text
                FROM path AS child
                INNER JOIN outlines AS parent ON parent.id = child.parent_id
            )
            SELECT DISTINCT id, parent_id, text FROM path;
        "#,
        parent_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, Ancestor>(&query);

    for id in parent_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn y_updates_by_doc_id(
    pool: &SqlitePool,
    y_doc_id: UUIDv7Base64URL,
) -> Result<Vec<BytesBase64URL>> {
    let mut query_builder = sqlx::query_scalar::<_, BytesBase64URL>(
        r#"
            SELECT data
            FROM y_updates
            WHERE y_doc_id = ?;
        "#,
    );

    query_builder = query_builder.bind(y_doc_id);
    query_builder.fetch_all(pool).await.map_err(|e| e.into())
}

pub async fn y_updates_by_id(
    pool: &SqlitePool,
    y_update_ids: &[UUIDv7Base64URL],
) -> Result<Vec<YUpdate>> {
    let query = format!(
        r#"
            SELECT id, y_doc_id, data
            FROM y_updates
            WHERE id IN ({});
        "#,
        y_update_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, YUpdate>(&query);

    for id in y_update_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn cards_by_id(pool: &SqlitePool, card_ids: &[UUIDv7Base64URL]) -> Result<Vec<Card>> {
    let query = format!(
        r#"
            WITH c1 AS (
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                GROUP BY cards.id
                WHERE id IN ({}) AND is_deleted = false
                UNION
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                JOIN c1 ON cards.id = c1.quote_id
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE is_deleted = false
            )
            SELECT
                c1.id, c1.outline_id, c1.fractional_index, c1.doc, c1.quote_id, c1.quote_version_id,
                jsonb_group_array(outline_paths.path) AS links,
                c1.created_at,
                c1.updated_at
            FROM c1
            LEFT JOIN card_links ON c1.id = card_links.id_from
            LEFT JOIN outline_paths ON card_links.id_to = outline_paths.outline_id
            GROUP BY id;
        "#,
        card_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawCard>(&query);

    for id in card_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
        .map_err(eyre::Error::from)
}

pub async fn cards_for_index_by_id(
    pool: &SqlitePool,
    card_ids: &[UUIDv7Base64URL],
) -> Result<Vec<CardForIndex>> {
    let query = format!(
        r#"
            WITH c1 AS (
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                GROUP BY cards.id
                WHERE id IN ({}) AND is_deleted = false
                UNION
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                JOIN c1 ON cards.id = c1.quote_id
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE is_deleted = false
            )
            SELECT
                c1.id,
                y_docs.pot_id,
                c1.outline_id,
                c1.fractional_index,
                c1.doc, c1.quote_id,
                c1.quote_version_id,
                path.path,
                jsonb_group_array(links.path) AS links,
                c1.created_at,
                c1.updated_at
            FROM c1
            INNER JOIN y_docs ON c1.id = y_docs.id
            LEFT JOIN card_links ON c1.id = card_links.id_from
            LEFT JOIN outline_paths AS path ON c1.outline_id = path.outline_id
            LEFT JOIN outline_paths AS links ON card_links.id_to = links.outline_id
            GROUP BY id;
        "#,
        card_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawCardForIndex>(&query);

    for id in card_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_cards| raw_cards.into_iter().map(CardForIndex::from).collect())
        .map_err(eyre::Error::from)
}

pub async fn cards_by_outline_id(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Card>> {
    let query = format!(
        r#"
            WITH c1 AS (
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE cards.outline_id IN ({}) AND is_deleted = false
                UNION
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                    quotes.quote_id AS quote_id,
                    quotes.version_id AS quote_version_id,
                    cards.created_at,
                    cards.updated_at
                FROM cards
                JOIN c1 ON cards.id = c1.quote_id
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE is_deleted = false
            )
            SELECT
                c1.id, c1.outline_id, c1.fractional_index, c1.doc, c1.quote_id, c1.quote_version_id,
                jsonb_group_array(outline_paths.path) AS links,
                c1.created_at,
                c1.updated_at
            FROM c1
            LEFT JOIN card_links ON c1.id = card_links.id_from
            LEFT JOIN outline_paths ON card_links.id_to = outline_paths.outline_id
            GROUP BY id;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawCard>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
        .map_err(eyre::Error::from)
}

pub async fn cards_by_created_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<Card>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();

    let query = r#"
        WITH c1 AS (
            SELECT
                cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                quotes.quote_id AS quote_id,
                quotes.version_id AS quote_version_id,
                cards.created_at,
                cards.updated_at
            FROM cards
            LEFT JOIN quotes ON cards.id = quotes.card_id
            WHERE ? <= created_at AND created_at < ? AND is_deleted = false
            UNION
            SELECT
                cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                quotes.quote_id AS quote_id,
                quotes.version_id AS quote_version_id,
                cards.created_at,
                cards.updated_at
            FROM cards
            LEFT JOIN quotes ON cards.id = quotes.card_id
            INNER JOIN c1 ON cards.id = c1.quote_id
            WHERE is_deleted = false
            ORDER BY created_at DESC
        )
        SELECT
            c1.id, c1.outline_id, c1.fractional_index, c1.doc, c1.quote_id, c1.quote_version_id,
            jsonb_group_array(outline_paths.path) AS links,
            c1.created_at,
            c1.updated_at
        FROM c1
        LEFT JOIN card_links ON c1.id = card_links.id_from
        LEFT JOIN outline_paths ON card_links.id_to = outline_paths.outline_id
        GROUP BY id;
    "#;

    let mut query_builder = sqlx::query_as::<_, RawCard>(query);

    query_builder = query_builder.bind(from);
    query_builder = query_builder.bind(to);

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
        .map_err(eyre::Error::from)
}

pub async fn card_delete_targets(
    pool: &SqlitePool,
    deleted_ids: &[UUIDv7Base64URL],
) -> eyre::Result<Vec<DeleteTarget>> {
    let query = format!(
        r#"
            SELECT id, pot_id
            FROM cards
            WHERE id IN ({});
        "#,
        &deleted_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, DeleteTarget>(&query);

    for id in deleted_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| e.into())
}

pub async fn conflicting_outline_ids(
    pool: &SqlitePool,
    outline_id: UUIDv7Base64URL,
    parent_id: Option<UUIDv7Base64URL>,
    text: &str,
) -> Result<Vec<(UUIDv7Base64URL, String)>> {
    #[derive(FromRow)]
    struct QueryResult {
        id: UUIDv7Base64URL,
        text: String,
    }

    sqlx::query_as::<_, QueryResult>(
        r#"
            WITH ConflictingOutlines AS (
                SELECT text
                FROM outlines
                WHERE
                    id != ?
                    AND (parent_id = ? OR (? IS NULL AND parent_id IS NULL))
                GROUP BY text
                HAVING COUNT(*) > 1
            )
            SELECT id, text
            FROM outlines
            WHERE
                (text = ? OR text IN (SELECT text FROM ConflictingOutlines))
                AND id != ?
                AND (parent_id = ? OR (? IS NULL AND parent_id IS NULL));
        "#,
    )
    .bind(outline_id)
    .bind(parent_id)
    .bind(parent_id)
    .bind(text)
    .bind(outline_id)
    .bind(parent_id)
    .bind(parent_id)
    .fetch_all(pool)
    .await
    .map(|r| r.into_iter().map(|r| (r.id, r.text)).collect())
    .map_err(|e| e.into())
}

pub async fn descendant_ids(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    include_cards: bool,
) -> Result<(Vec<UUIDv7Base64URL>, Vec<UUIDv7Base64URL>)> {
    let outline_ids = {
        let query = format!(
            r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT id
                    FROM outlines
                    WHERE id IN ({}) AND is_deleted = false
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

        let mut query_builder = sqlx::query_scalar::<_, UUIDv7Base64URL>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
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

        let mut query_builder = sqlx::query_scalar::<_, UUIDv7Base64URL>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    } else {
        vec![]
    };

    Ok((outline_ids, card_ids))
}

pub async fn outline_trees(
    pool: &SqlitePool,
    root_ids: &[UUIDv7Base64URL],
    depth: Option<u32>,
) -> Result<Vec<Outline>> {
    match depth {
        Some(depth) => {
            let query = format!(
                r#"
                    WITH RECURSIVE outline_tree AS (
                        SELECT
                            id, parent_id, fractional_index, doc, text, 0 AS depth, created_at, updated_at
                        FROM outlines
                        WHERE id IN ({}) AND is_deleted = false
                        UNION ALL
                        SELECT
                            child.id, child.parent_id, child.fractional_index, child.doc,
                            child.text,
                            parent.depth + 1 AS depth, child.created_at, child.updated_at
                        FROM outline_tree AS parent
                        JOIN outlines AS child ON parent.id = child.parent_id
                        WHERE child.is_deleted = false AND depth <= ?
                    )
                    SELECT
                        outline_tree.id,
                        outline_tree.parent_id,
                        outline_tree.fractional_index,
                        outline_tree.doc,
                        outline_tree.text,
                        jsonb_group_array(outline_paths.path) AS links,
                        outline_tree.created_at,
                        outline_tree.updated_at
                    FROM outline_tree
                    LEFT JOIN outline_links ON outline_tree.id = outline_links.id_from
                    LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                    GROUP BY id;
                "#,
                root_ids
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            let mut query_builder = sqlx::query_as::<_, Outline>(&query);

            for id in root_ids {
                query_builder = query_builder.bind(id);
            }

            query_builder = query_builder.bind(depth);

            query_builder
            .fetch_all(pool)
            .await
        }
        None => {
            let query = format!(
                r#"
                    WITH RECURSIVE outline_tree AS (
                        SELECT
                            id, parent_id, fractional_index, doc, text, created_at, updated_at
                        FROM outlines
                        WHERE id IN ({}) AND is_deleted = false
                        UNION ALL
                        SELECT
                            child.id, child.parent_id, child.fractional_index, child.doc, child.text,
                            child.created_at, child.updated_at
                        FROM outline_tree AS parent
                        JOIN outlines AS child ON parent.id = child.parent_id
                        WHERE child.is_deleted = false
                    )
                    SELECT
                        outline_tree.id,
                        outline_tree.parent_id,
                        outline_tree.fractional_index,
                        outline_tree.doc,
                        outline_tree.text,
                        jsonb_group_array(outline_paths.path) AS links,
                        outline_tree.created_at,
                        outline_tree.updated_at
                    FROM outline_tree
                    LEFT JOIN outline_links ON outline_tree.id = outline_links.id_from
                    LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                    GROUP BY id;
                "#,
                root_ids
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            let mut query_builder = sqlx::query_as::<_, Outline>(&query);

            for id in root_ids {
                query_builder = query_builder.bind(id);
            }

            query_builder
            .fetch_all(pool)
            .await
        }
    }
        .map_err(eyre::Error::from)
}

pub async fn outlines_by_id(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Outline>> {
    let query = format!(
        r#"
            SELECT
                id,
                parent_id,
                fractional_index,
                doc,
                text,
                jsonb_group_array(path) AS links,
                created_at,
                updated_at
            FROM outlines
            LEFT JOIN outline_links ON outlines.id = outline_links.id_from
            LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
            WHERE id IN ({}) AND is_deleted = false
            GROUP BY id;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, Outline>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn outlines_for_index_by_id(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<OutlineForIndex>> {
    let query = format!(
        r#"
            SELECT
                id,
                y_docs.pot_id,
                parent_id,
                fractional_index,
                doc,
                text,
                outline_paths.path,
                jsonb_group_array(links.path) AS links,
                created_at,
                updated_at
            FROM outlines
            INNER JOIN y_docs ON outlines.id = y_docs.id
            LEFT JOIN outline_links ON outlines.id = outline_links.id_from
            LEFT JOIN outline_paths AS path ON path.outline_id = outlines.id
            LEFT JOIN outline_paths AS links ON links.outline_id = outline_links.id_to
            GROUP BY id
            WHERE id IN ({}) AND is_deleted = false;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, OutlineForIndex>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn outline_delete_targets(
    pool: &SqlitePool,
    deleted_ids: &[UUIDv7Base64URL],
) -> eyre::Result<Vec<DeleteTarget>> {
    let query = format!(
        r#"
            SELECT id, pot_id
            FROM outlines
            WHERE id IN ({});
        "#,
        &deleted_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, DeleteTarget>(&query);

    for id in deleted_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| e.into())
}

pub async fn relation_back(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    card_ids: &[UUIDv7Base64URL],
) -> Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    id,
                    parent_id,
                    fractional_index,
                    doc,
                    text,
                    jsonb_group_array(path) AS links,
                    created_at,
                    updated_at
                FROM outlines
                LEFT JOIN outline_links ON outlines.id = outline_links.id_from
                LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                WHERE outlines.is_deleted = false AND id_to IN ({})
                GROUP BY id;
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
                        cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                        quotes.quote_id AS quote_id,
                        quotes.version_id AS quote_version_id,
                        cards.created_at,
                        cards.updated_at
                    FROM card_links
                    INNER JOIN cards ON card_links.id_from = cards.id
                    LEFT JOIN quotes ON quotes.card_id = cards.id
                    WHERE card_links.id_to IN ({}) AND cards.is_deleted = false
                    UNION
                    SELECT
                        cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                        quotes.quote_id AS quote_id,
                        quotes.version_id AS quote_version_id,
                        cards.created_at,
                        cards.updated_at
                    FROM quotes
                    INNER JOIN cards ON quotes.card_id = cards.id
                    WHERE quotes.quote_id IN ({}) AND cards.is_deleted = false
                    UNION
                    SELECT
                        cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                        quotes.quote_id AS quote_id,
                        quotes.version_id AS quote_version_id,
                        cards.created_at,
                        cards.updated_at
                    FROM cards
                    INNER JOIN c1 ON c1.quote_id = cards.id
                    INNER JOIN quotes ON quotes.card_id = cards.id
                    WHERE cards.is_deleted = false
                )
                SELECT
                    c1.id, c1.outline_id, c1.fractional_index, c1.doc, c1.quote_id, c1.quote_version_id,
                    jsonb_group_array(outline_paths.path) AS links,
                    c1.created_at,
                    c1.updated_at
                FROM c1
                LEFT JOIN card_links ON c1.id = card_links.id_from
                LEFT JOIN outline_paths ON card_links.id_to = outline_paths.outline_id
                GROUP BY id;
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

pub async fn relation_forward(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    card_ids: &[UUIDv7Base64URL],
) -> Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    id,
                    parent_id,
                    fractional_index,
                    doc,
                    text,
                    jsonb_group_array(path) AS links,
                    created_at,
                    updated_at
                FROM outlines
                LEFT JOIN outline_links ON outlines.id = outline_links.id_from
                LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                WHERE outline_links.id_from IN ({}) AND outlines.is_deleted = false
                GROUP BY id;
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
                        cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                        q2.quote_id AS quote_id,
                        q2.version_id AS quote_version_id,
                        cards.created_at,
                        cards.updated_at
                    FROM cards
                    INNER JOIN quotes AS q1 ON q1.quote_id = cards.id
                    LEFT JOIN quotes AS q2 ON q2.card_id = cards.id
                    WHERE q1.card_id IN ({}) AND cards.is_deleted = false
                    UNION
                    SELECT
                        cards.id, cards.outline_id, cards.fractional_index, cards.doc,
                        quotes.quote_id AS quote_id,
                        quotes.version_id AS quote_version_id,
                        cards.created_at,
                        cards.updated_at
                    FROM cards
                    INNER JOIN c1 ON c1.quote_id = cards.id
                    LEFT JOIN quotes ON quotes.card_id = cards.id
                    WHERE cards.is_deleted = false
                )
                SELECT
                    c1.id, c1.outline_id, c1.fractional_index, c1.doc, c1.quote_id, c1.quote_version_id,
                    jsonb_group_array(outline_paths.path) AS links,
                    c1.created_at,
                    c1.updated_at
                FROM c1
                LEFT JOIN card_links ON c1.id = card_links.id_from
                LEFT JOIN outline_paths ON card_links.id_to = outline_paths.outline_id
                GROUP BY id;
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

pub async fn relation_count(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    card_ids: &[UUIDv7Base64URL],
) -> Result<Vec<LinkCount>> {
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
                    WHERE quote_id = this.id
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

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn recursive_relation_count(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    card_ids: &[UUIDv7Base64URL],
) -> Result<Vec<LinkCount>> {
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
                        WHERE quote_id
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
                    WHERE quote_id = this.id
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

    query_builder
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn unversioned_y_updates(pool: &SqlitePool) -> Result<Vec<YUpdate>> {
    sqlx::query_as::<_, YUpdate>(
        r#"
            SELECT id, y_doc_id, data
            FROM y_updates
            WHERE version_id IS NULL;
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(eyre::Error::from)
}

pub async fn oplog_rowids_all(pool: &SqlitePool) -> Result<Vec<i64>> {
    sqlx::query_scalar::<_, i64>("SELECT rowid FROM operation_logs;")
        .fetch_all(pool)
        .await
        .map_err(eyre::Error::from)
}

pub async fn oplogs_by_rowid(pool: &SqlitePool, rowids: &[i64]) -> Result<Vec<Oplog>> {
    let query = format!(
        r#"
            SELECT * FROM operation_logs WHERE rowid IN ({});
        "#,
        rowids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, Oplog>(&query);

    for id in rowids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| e.into())
}
