use crate::{
    search_engine::DeleteTarget,
    types::{
        model::{
            Ancestor, Descendant, LinkCount, Oplog, Outline, OutlineForIndex, Paragraph,
            ParagraphForIndex, Path, PendingYUpdate, Pot, RawParagraph, RawParagraphForIndex,
            YUpdate,
        },
        state::{AppState, WorkspaceState},
        util::{BytesBase64URL, UUIDv7Base64URL},
    },
};
use chrono::{DateTime, Utc};
use eyre::{Context, Result};
use sqlx::{prelude::FromRow, SqlitePool};

pub async fn pots(pool: &SqlitePool) -> Result<Vec<Pot>> {
    sqlx::query_as::<_, Pot>("SELECT * FROM pots;")
        .fetch_all(pool)
        .await
        .context("database error")
}

pub async fn pot_by_id(pool: &SqlitePool, pot_id: UUIDv7Base64URL) -> Result<Pot> {
    sqlx::query_as::<_, Pot>("SELECT * FROM pots WHERE id = ?;")
        .bind(pot_id)
        .fetch_one(pool)
        .await
        .context("database error")
}

pub async fn path(pool: &SqlitePool, outline_id: UUIDv7Base64URL) -> Result<Path> {
    let query = r#"
        SELECT path
        FROM outline_paths
        WHERE outline_id = ?;
    "#;

    let mut query_builder = sqlx::query_scalar::<_, Path>(query);

    query_builder = query_builder.bind(outline_id);

    query_builder
        .fetch_one(pool)
        .await
        .context("database error")
}

pub async fn self_and_its_ancestors(
    pool: &SqlitePool,
    parent_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Ancestor>> {
    let query = format!(
        r#"
            WITH RECURSIVE ancestors AS (
                SELECT
                    id, parent_id, text, hidden
                FROM outlines
                WHERE id IN ({})
                UNION
                SELECT
                    parent.id, parent.parent_id, parent.text, parent.hidden
                FROM outlines AS child
                INNER JOIN outlines AS parent ON parent.id = child.parent_id
            )
            SELECT id, parent_id, text, hidden FROM ancestors;
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
        .context("database error")
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
    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
}

pub async fn y_updates_by_id(
    pool: &SqlitePool,
    y_update_ids: &[UUIDv7Base64URL],
) -> Result<Vec<YUpdate>> {
    let query = format!(
        r#"
            SELECT id, y_doc_id, data, timestamp, version_id
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
        .context("database error")
}

pub async fn paragraphs_by_id(
    pool: &SqlitePool,
    paragraph_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Paragraph>> {
    let query = format!(
        r#"
            SELECT
                paragraphs.id,
                paragraphs.outline_id,
                paragraphs.fractional_index,
                paragraphs.doc,
                quotes.quoted_paragraph_id AS quoted_paragraph_id,
                quotes.version_id AS quoted_version_id,
                quotes.doc AS quoted_doc,
                quoted_paragraphs.doc AS latest_quoted_doc,
                quoted_paths.path AS quoted_path,
                jsonb_group_array(outline_paths.path) AS links,
                paragraphs.hidden,
                paragraphs.deleted,
                paragraphs.created_at,
                paragraphs.updated_at
            FROM paragraphs
            LEFT JOIN quotes ON paragraphs.id = quotes.paragraph_id
            LEFT JOIN paragraphs AS quoted_paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
            LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
            LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
            LEFT JOIN outline_paths ON paragraph_links.id_to = outline_paths.outline_id
            WHERE id IN ({}) AND paragraphs.deleted = false
            GROUP BY paragraphs.id;
        "#,
        paragraph_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawParagraph>(&query);

    for id in paragraph_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_paragraphs| raw_paragraphs.into_iter().map(Paragraph::from).collect())
        .context("database error")
}

pub async fn paragraphs_for_index_by_id(
    pool: &SqlitePool,
    paragraph_ids: &[UUIDv7Base64URL],
) -> Result<Vec<ParagraphForIndex>> {
    let query = format!(
        r#"
            SELECT
                paragraphs.id,
                y_docs.pot_id,
                paragraphs.outline_id,
                paragraphs.fractional_index,
                paragraphs.doc,
                quotes.quoted_paragraph_id AS quoted_paragraph_id,
                quotes.version_id AS quoted_version_id,
                quotes.doc AS quoted_doc,
                quoted_paragraphs.doc AS latest_quoted_doc,
                quoted_paths.path AS quoted_path,
                COALESCE(
                    path.path,
                    jsonb_array()
                ) AS path,
                jsonb_group_array(links.path) AS links,
                paragraphs.hidden,
                paragraphs.deleted,
                paragraphs.created_at,
                paragraphs.updated_at
            FROM paragraphs
            LEFT JOIN y_docs ON paragraphs.id = y_docs.id
            LEFT JOIN quotes ON paragraphs.id = quotes.paragraph_id
            LEFT JOIN paragraphs AS quoted_paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
            LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
            LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
            LEFT JOIN outline_paths AS path ON paragraphs.outline_id = path.outline_id
            LEFT JOIN outline_paths AS links ON paragraph_links.id_to = links.outline_id
            WHERE paragraphs.id IN ({}) AND paragraphs.deleted = false
            GROUP BY paragraphs.id;
        "#,
        paragraph_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawParagraphForIndex>(&query);

    for id in paragraph_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_paragraphs| {
            raw_paragraphs
                .into_iter()
                .map(ParagraphForIndex::from)
                .collect()
        })
        .context("database error")
}

pub async fn paragraphs_by_outline_id(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Paragraph>> {
    let query = format!(
        r#"
            SELECT
                paragraphs.id,
                paragraphs.outline_id,
                paragraphs.fractional_index,
                paragraphs.doc,
                quotes.quoted_paragraph_id AS quoted_paragraph_id,
                quotes.version_id AS quoted_version_id,
                quotes.doc AS quoted_doc,
                quoted_paragraphs.doc AS latest_quoted_doc,
                quoted_paths.path AS quoted_path,
                jsonb_group_array(outline_paths.path) AS links,
                paragraphs.hidden,
                paragraphs.deleted,
                paragraphs.created_at,
                paragraphs.updated_at
            FROM paragraphs
            LEFT JOIN quotes ON paragraphs.id = quotes.paragraph_id
            LEFT JOIN paragraphs AS quoted_paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
            LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
            LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
            LEFT JOIN outline_paths ON paragraph_links.id_to = outline_paths.outline_id
            WHERE paragraphs.outline_id IN ({}) AND paragraphs.deleted = false
            GROUP BY paragraphs.id;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawParagraph>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_paragraphs| raw_paragraphs.into_iter().map(Paragraph::from).collect())
        .context("database error")
}

pub async fn paragraphs_by_created_at(
    pool: &SqlitePool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<Paragraph>> {
    let from = from.timestamp_millis();
    let to = to.timestamp_millis();

    let query = r#"
        SELECT
            paragraphs.id,
            paragraphs.outline_id,
            paragraphs.fractional_index,
            paragraphs.doc,
            quotes.quoted_paragraph_id AS quoted_paragraph_id,
            quotes.version_id AS quoted_version_id,
            quotes.doc AS quoted_doc,
            quoted_paragraphs.doc AS latest_quoted_doc,
            quoted_paths.path AS quoted_path,
            jsonb_group_array(outline_paths.path) AS links,
            paragraphs.hidden,
            paragraphs.deleted,
            paragraphs.created_at,
            paragraphs.updated_at
        FROM paragraphs
        LEFT JOIN quotes ON paragraphs.id = quotes.paragraph_id
        LEFT JOIN paragraphs AS quoted_paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
        LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
        LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
        LEFT JOIN outline_paths ON paragraph_links.id_to = outline_paths.outline_id
        WHERE ? <= paragraphs.created_at AND paragraphs.created_at < ? AND paragraphs.deleted = false
        GROUP BY paragraphs.id
        ORDER BY paragraphs.created_at DESC;
    "#;

    let mut query_builder = sqlx::query_as::<_, RawParagraph>(query);

    query_builder = query_builder.bind(from);
    query_builder = query_builder.bind(to);

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_paragraphs| raw_paragraphs.into_iter().map(Paragraph::from).collect())
        .context("database error")
}

pub async fn paragraph_delete_targets(
    pool: &SqlitePool,
    deleted_ids: &[UUIDv7Base64URL],
) -> eyre::Result<Vec<DeleteTarget>> {
    let query = format!(
        r#"
            SELECT id, pot_id
            FROM paragraphs
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

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
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
    .context("database error")
}

pub async fn descendant_ids(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    include_paragraphs: bool,
) -> Result<(Vec<UUIDv7Base64URL>, Vec<UUIDv7Base64URL>)> {
    let outline_ids = {
        let query = format!(
            r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT id
                    FROM outlines
                    WHERE id IN ({}) AND deleted = false
                    UNION ALL
                    SELECT child.id
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.deleted = false
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

    let paragraph_ids = if include_paragraphs {
        let query = format!(
            r#"
                SELECT id FROM paragraphs WHERE outline_id IN ({}) AND deleted = false;
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

    Ok((outline_ids, paragraph_ids))
}

pub async fn descendants(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<Descendant>> {
    let query = format!(
        r#"
            WITH RECURSIVE descendants AS (
                SELECT
                    id,
                    parent_id
                FROM outlines
                WHERE parent_id IN ({})
                UNION ALL
                SELECT
                    child.id,
                    child.parent_id
                FROM outlines AS child
                INNER JOIN descendants AS parent ON parent.id = child.parent_id
            )
            SELECT 
                id,
                parent_id,
                path
            FROM descendants
            INNER JOIN outline_paths ON outline_paths.outline_id = descendants.id;
        "#,
        outline_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, Descendant>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
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
                            id,
                            parent_id,
                            fractional_index,
                            doc,
                            text,
                            0 AS depth,
                            hidden,
                            collapsed,
                            deleted,
                            created_at,
                            updated_at
                        FROM outlines
                        WHERE id IN ({}) AND deleted = false
                        UNION ALL
                        SELECT
                            child.id,
                            child.parent_id,
                            child.fractional_index,
                            child.doc,
                            child.text,
                            parent.depth + 1 AS depth,
                            child.hidden,
                            child.collapsed,
                            child.deleted,
                            child.created_at,
                            child.updated_at
                        FROM outline_tree AS parent
                        JOIN outlines AS child ON parent.id = child.parent_id
                        WHERE child.deleted = false AND depth <= ?
                    )
                    SELECT
                        outline_tree.id,
                        outline_tree.parent_id,
                        outline_tree.fractional_index,
                        outline_tree.doc,
                        outline_tree.text,
                        jsonb_group_array(outline_paths.path) AS links,
                        outline_tree.hidden,
                        outline_tree.collapsed,
                        outline_tree.deleted,
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

            query_builder.fetch_all(pool).await
        }
        None => {
            let query = format!(
                r#"
                    WITH RECURSIVE outline_tree AS (
                        SELECT
                            id,
                            parent_id,
                            fractional_index,
                            doc,
                            text,
                            hidden,
                            collapsed,
                            deleted,
                            created_at,
                            updated_at
                        FROM outlines
                        WHERE id IN ({}) AND deleted = false
                        UNION ALL
                        SELECT
                            child.id,
                            child.parent_id,
                            child.fractional_index,
                            child.doc,
                            child.text,
                            child.hidden,
                            child.collapsed,
                            child.deleted,
                            child.created_at,
                            child.updated_at
                        FROM outline_tree AS parent
                        JOIN outlines AS child ON parent.id = child.parent_id
                        WHERE child.deleted = false
                    )
                    SELECT
                        outline_tree.id,
                        outline_tree.parent_id,
                        outline_tree.fractional_index,
                        outline_tree.doc,
                        outline_tree.text,
                        outline_tree.hidden,
                        outline_tree.collapsed,
                        outline_tree.deleted,
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

            query_builder.fetch_all(pool).await
        }
    }
    .context("database error")
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
                hidden,
                collapsed,
                deleted,
                created_at,
                updated_at
            FROM outlines
            LEFT JOIN outline_links ON outlines.id = outline_links.id_from
            LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
            WHERE id IN ({}) AND deleted = false
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
        .context("database error")
}

pub async fn outlines_for_index_by_id(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
) -> Result<Vec<OutlineForIndex>> {
    let query = format!(
        r#"
            SELECT
                outlines.id,
                y_docs.pot_id,
                outlines.parent_id,
                outlines.fractional_index,
                outlines.doc,
                outlines.text,
                COALESCE(
                    path.path,
                    jsonb_array()
                ) AS path,
                jsonb_group_array(links.path) AS links,
                outlines.hidden,
                outlines.collapsed,
                outlines.deleted,
                outlines.created_at,
                outlines.updated_at
            FROM outlines
            INNER JOIN y_docs ON outlines.id = y_docs.id
            LEFT JOIN outline_links ON outlines.id = outline_links.id_from
            LEFT JOIN outline_paths AS path ON path.outline_id = outlines.id
            LEFT JOIN outline_paths AS links ON links.outline_id = outline_links.id_to
            WHERE outlines.id IN ({}) AND deleted = false
            GROUP BY outlines.id;
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
        .context("database error")
}

pub async fn outline_delete_targets(
    pool: &SqlitePool,
    deleted_ids: &[UUIDv7Base64URL],
) -> eyre::Result<Vec<DeleteTarget>> {
    let query = format!(
        r#"
            SELECT outlines.id, y_docs.pot_id
            FROM outlines
            INNER JOIN y_docs ON outlines.id = y_docs.id
            WHERE outlines.id IN ({});
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

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
}

pub async fn relation_back(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    paragraph_ids: &[UUIDv7Base64URL],
) -> Result<(Vec<Outline>, Vec<Paragraph>)> {
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
                    hidden,
                    collapsed,
                    deleted,
                    created_at,
                    updated_at
                FROM outlines
                LEFT JOIN outline_links ON outlines.id = outline_links.id_from
                LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                WHERE outlines.deleted = false AND id_to IN ({})
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

    let paragraphs: Vec<Paragraph> = {
        let query = format!(
            r#"
                SELECT
                    paragraphs.id,
                    paragraphs.outline_id,
                    paragraphs.fractional_index,
                    paragraphs.doc,
                    quotes.quoted_paragraph_id AS quoted_paragraph_id,
                    quotes.version_id AS quoted_version_id,
                    quotes.doc AS quoted_doc,
                    quoted_paragraphs.doc AS latest_quoted_doc,
                    quoted_paths.path AS quoted_path,
                    jsonb_group_array(outline_paths.path) AS links,
                    paragraphs.hidden,
                    paragraphs.deleted,
                    paragraphs.created_at,
                    paragraphs.updated_at
                FROM paragraph_links
                INNER JOIN paragraphs ON paragraph_links.id_from = paragraphs.id
                LEFT JOIN quotes ON quotes.paragraph_id = paragraphs.id
                LEFT JOIN paragraphs AS quoted_paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
                LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
                LEFT JOIN paragraph_links AS links_to ON paragraphs.id = links_to.id_from
                LEFT JOIN outline_paths ON links_to.id_to = outline_paths.outline_id
                WHERE paragraph_links.id_to IN ({}) AND paragraphs.deleted = false
                GROUP BY paragraphs.id
                UNION ALL
                SELECT
                    paragraphs.id,
                    paragraphs.outline_id,
                    paragraphs.fractional_index,
                    paragraphs.doc,
                    quotes_to.quoted_paragraph_id AS quoted_paragraph_id,
                    quotes_to.version_id AS quoted_version_id,
                    quotes_to.doc AS quoted_doc,
                    quoted_paragraphs.doc AS latest_quoted_doc,
                    quoted_paths.path AS quoted_path,
                    jsonb_group_array(outline_paths.path) AS links,
                    paragraphs.hidden,
                    paragraphs.deleted,
                    paragraphs.created_at,
                    paragraphs.updated_at
                FROM quotes
                INNER JOIN paragraphs ON quotes.paragraph_id = paragraphs.id
                LEFT JOIN quotes AS quotes_to ON quotes_to.paragraph_id = paragraphs.id
                LEFT JOIN paragraphs AS quoted_paragraphs ON quotes_to.quoted_paragraph_id = paragraphs.id
                LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
                LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
                LEFT JOIN outline_paths ON paragraph_links.id_to = outline_paths.outline_id
                WHERE quotes.quoted_paragraph_id IN ({}) AND paragraphs.deleted = false
                GROUP BY paragraphs.id;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
            paragraph_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
        );

        let mut query_builder = sqlx::query_as::<_, RawParagraph>(&query);

        for id in outline_ids.iter().chain(paragraph_ids.iter()) {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .fetch_all(pool)
            .await
            .map(|raw_paragraphs| raw_paragraphs.into_iter().map(Paragraph::from).collect())?
    };

    Ok((outlines, paragraphs))
}

pub async fn relation_forward(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    paragraph_ids: &[UUIDv7Base64URL],
) -> Result<(Vec<Outline>, Vec<Paragraph>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    outlines.id,
                    outlines.parent_id,
                    outlines.fractional_index,
                    outlines.doc,
                    outlines.text,
                    jsonb_group_array(path) AS links,
                    outlines.hidden,
                    outlines.collapsed,
                    outlines.deleted,
                    outlines.created_at,
                    outlines.updated_at
                FROM outlines
                INNER JOIN outline_links ON outlines.id = outline_links.id_to
                LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                WHERE outline_links.id_from IN ({}) AND outlines.deleted = false
                GROUP BY id
                UNION
                SELECT
                    outlines.id,
                    outlines.parent_id,
                    outlines.fractional_index,
                    outlines.doc,
                    outlines.text,
                    jsonb_group_array(path) AS links,
                    outlines.hidden,
                    outlines.collapsed,
                    outlines.deleted,
                    outlines.created_at,
                    outlines.updated_at
                FROM outlines
                INNER JOIN paragraph_links ON paragraph_links.id_to = outlines.id
                LEFT JOIN outline_links ON outlines.id = outline_links.id_to
                LEFT JOIN outline_paths ON outline_paths.outline_id = outline_links.id_to
                WHERE paragraph_links.id_from IN ({}) AND outlines.deleted = false
                GROUP BY id;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
            paragraph_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for id in outline_ids.iter().chain(paragraph_ids.iter()) {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    let paragraphs: Vec<Paragraph> = {
        let query = format!(
            r#"
                SELECT
                    paragraphs.id,
                    paragraphs.outline_id,
                    paragraphs.fractional_index,
                    paragraphs.doc,
                    quotes_to.quoted_paragraph_id AS quoted_paragraph_id,
                    quotes_to.version_id AS quoted_version_id,
                    quotes_to.doc AS quoted_doc,
                    quoted_paragraphs.doc AS latest_quoted_doc,
                    quoted_paths.path AS quoted_path,
                    jsonb_group_array(outline_paths.path) AS links,
                    paragraphs.hidden,
                    paragraphs.deleted,
                    paragraphs.created_at,
                    paragraphs.updated_at
                FROM quotes
                INNER JOIN paragraphs ON quotes.quoted_paragraph_id = paragraphs.id
                LEFT JOIN quotes AS quotes_to ON quotes_to.paragraph_id = paragraphs.id
                LEFT JOIN paragraphs AS quoted_paragraphs ON quotes_to.quoted_paragraph_id = paragraphs.id
                LEFT JOIN outline_paths AS quoted_paths ON quoted_paragraphs.outline_id = quoted_paths.outline_id
                LEFT JOIN paragraph_links ON paragraphs.id = paragraph_links.id_from
                LEFT JOIN outline_paths ON paragraph_links.id_to = outline_paths.outline_id
                WHERE paragraphs.id IN ({}) AND paragraphs.deleted = false
                GROUP BY paragraphs.id;
            "#,
            paragraph_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, RawParagraph>(&query);

        for id in paragraph_ids {
            query_builder = query_builder.bind(id);
        }

        query_builder
            .fetch_all(pool)
            .await
            .map(|raw_paragraphs| raw_paragraphs.into_iter().map(Paragraph::from).collect())?
    };

    Ok((outlines, paragraphs))
}

pub async fn relation_count(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    paragraph_ids: &[UUIDv7Base64URL],
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
                        FROM paragraph_links
                        WHERE paragraph_links.id_to = this.id
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
                    WHERE quoted_paragraph_id = this.id
                ) AS back,
                (
                    (
                        SELECT COUNT(*)
                        FROM paragraph_links
                        WHERE paragraph_links.id_from = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE paragraph_id = this.id
                    )
                ) AS forward
            FROM paragraphs AS this
            WHERE id IN ({});
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
        paragraph_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
    );

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in paragraph_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
}

pub async fn recursive_relation_count(
    pool: &SqlitePool,
    outline_ids: &[UUIDv7Base64URL],
    paragraph_ids: &[UUIDv7Base64URL],
) -> Result<Vec<LinkCount>> {
    let query = format!(
        r#"
            WITH RECURSIVE tree AS (
                SELECT id, id AS root_id
                FROM outlines
                WHERE id IN ({}) AND deleted = false
                UNION ALL
                SELECT child.id, parent.root_id AS root_id
                FROM tree AS parent
                JOIN outlines AS child ON parent.id = child.parent_id
                WHERE child.deleted = false
            ),
            tree_paragraphs AS (
                SELECT paragraphs.id, tree.root_id
                FROM paragraphs
                INNER JOIN tree ON paragraphs.outline_id = tree.id
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
                        FROM paragraph_links
                        WHERE paragraph_links.id_to IN ((
                            SELECT id
                            FROM tree
                            WHERE tree.root_id = this.id
                        ))
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE quoted_paragraph_id
                         IN ((
                            SELECT id
                            FROM tree_paragraphs
                            WHERE tree_paragraphs.root_id = this.id
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
                        FROM paragraph_links
                        WHERE paragraph_links.id_from IN ((
                            SELECT id
                            FROM tree_paragraphs
                            WHERE tree_paragraphs.root_id = this.id
                        ))
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE
                            paragraph_id IN ((
                                SELECT id
                                FROM tree_paragraphs
                                WHERE tree_paragraphs.root_id = this.id
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
                    WHERE quoted_paragraph_id = this.id
                ) AS back,
                (
                    (
                        SELECT COUNT(*)
                        FROM paragraph_links
                        WHERE paragraph_links.id_from = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM quotes
                        WHERE paragraph_id = this.id
                    )
                ) AS forward
            FROM tree_paragraphs AS this
            WHERE id IN ({});
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
        paragraph_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
    );

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in paragraph_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
}

pub async fn unversioned_y_updates(pool: &SqlitePool) -> Result<Vec<YUpdate>> {
    sqlx::query_as::<_, YUpdate>(
        r#"
            SELECT id, y_doc_id, data, timestamp, version_id
            FROM y_updates
            WHERE version_id IS NULL;
        "#,
    )
    .fetch_all(pool)
    .await
    .context("database error")
}

pub async fn oplog_rowids_all(pool: &SqlitePool) -> Result<Vec<i64>> {
    sqlx::query_scalar::<_, i64>("SELECT rowid FROM operation_logs;")
        .fetch_all(pool)
        .await
        .context("database error")
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

    query_builder
        .fetch_all(pool)
        .await
        .context("database error")
}

#[allow(dead_code)]
pub async fn pending_y_updates(pool: &SqlitePool) -> Result<Vec<PendingYUpdate>> {
    sqlx::query_as::<_, PendingYUpdate>(
        r#"
            SELECT id, type AS doc_type, data, timestamp
            FROM pending_y_updates
            JOIN y_docs ON y_docs.id = pending_y_updates.y_doc_id;
        "#,
    )
    .fetch_all(pool)
    .await
    .context("database error")
}

pub async fn paragraphs_doc_and_path_by_id(
    pool: &SqlitePool,
    paragraph_id: UUIDv7Base64URL,
) -> Result<(String, Path)> {
    #[derive(FromRow)]
    struct QueryResult {
        doc: String,
        path: Path,
    }

    let query = r#"
        SELECT doc, path
        FROM paragraphs
        INNER JOIN outline_paths ON paragraphs.outline_id = outline_paths.outline_id
        WHERE id = ?;
    "#;

    sqlx::query_as::<_, QueryResult>(query)
        .bind(paragraph_id)
        .fetch_one(pool)
        .await
        .map(|r| (r.doc, r.path))
        .context("database error")
}

pub async fn app_state(pool: &SqlitePool) -> Result<Option<AppState>> {
    sqlx::query_scalar!(
        r#"
            SELECT value
            FROM kvs
            WHERE id = "app_state";
        "#
    )
    .fetch_optional(pool)
    .await?
    .map(|b| serde_sqlite_jsonb::from_slice::<AppState>(&b).context("database error"))
    .transpose()
}

pub async fn workspace_state(
    pool: &SqlitePool,
    pot_id: UUIDv7Base64URL,
) -> Result<Option<WorkspaceState>> {
    sqlx::query_scalar!(
        r#"
            SELECT value
            FROM workspaces
            WHERE pot_id = ?;
        "#,
        pot_id
    )
    .fetch_optional(pool)
    .await?
    .map(|b| serde_sqlite_jsonb::from_slice::<WorkspaceState>(&b).context("database error"))
    .transpose()
}
