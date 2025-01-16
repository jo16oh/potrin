use crate::types::{
    model::{Card, Outline},
    util::UUIDv7Base64URL,
};
use eyre::{OptionExt, Result};
use sqlx::SqliteExecutor;

pub async fn outline<'a, E>(conn: E, outline: &Outline) -> Result<i64>
where
    E: SqliteExecutor<'a>,
{
    let text = (!outline.text.is_empty()).then_some(&outline.text);

    sqlx::query_scalar!(
        r#"
            INSERT INTO outlines (
                id, parent_id, fractional_index, doc, text, created_at, updated_at, is_deleted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE
            SET
                parent_id = excluded.parent_id,
                fractional_index = excluded.fractional_index,
                doc = excluded.doc,
                updated_at = excluded.updated_at,
                is_deleted = excluded.is_deleted
            WHERE id = excluded.id
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        outline.id,
        outline.parent_id,
        outline.fractional_index,
        outline.doc,
        text,
        outline.created_at,
        outline.updated_at,
        0
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
}

pub async fn card<'a, E>(conn: E, card: &Card) -> Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar!(
        r#"
            INSERT INTO cards (
                id, outline_id, fractional_index, doc, created_at, updated_at, is_deleted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE
            SET
                outline_id = excluded.outline_id,
                fractional_index = excluded.fractional_index,
                doc = excluded.doc,
                updated_at = excluded.updated_at,
                is_deleted = excluded.is_deleted
            WHERE id = excluded.id
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        card.id,
        card.outline_id,
        card.fractional_index,
        card.doc,
        card.created_at,
        card.updated_at,
        0
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
}

pub async fn path<'a, E>(conn: E, values: &[(UUIDv7Base64URL, Vec<u8>)]) -> Result<()>
where
    E: SqliteExecutor<'a>,
{
    if values.is_empty() {
        return Ok(());
    }

    let query = format!(
        r#"
            INSERT INTO outline_paths (outline_id, path)
            VALUES {}
            ON CONFLICT
            DO UPDATE
            SET
                path = excluded.path;
        "#,
        values
            .iter()
            .map(|_| "(?, ?)")
            .collect::<Vec<_>>()
            .join(", ")
    );

    let query_builder = sqlx::query::<_>(&query);
    query_builder.execute(conn).await?;

    Ok(())
}
