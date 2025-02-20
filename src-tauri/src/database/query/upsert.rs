use crate::types::{
    model::{Outline, Paragraph},
    state::{AppState, WorkspaceState},
    util::UUIDv7Base64URL,
};
use eyre::{Context, Result};
use sqlx::SqliteExecutor;

pub async fn outline<'a, E>(conn: E, outline: &Outline) -> Result<i64>
where
    E: SqliteExecutor<'a>,
{
    let text = (!outline.text.is_empty()).then_some(&outline.text);

    sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO outlines (
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
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE
            SET
                parent_id = excluded.parent_id,
                fractional_index = excluded.fractional_index,
                doc = excluded.doc,
                text = excluded.text,
                hidden = excluded.hidden,
                collapsed = excluded.collapsed,
                deleted = excluded.deleted,
                updated_at = excluded.updated_at
            WHERE id = excluded.id
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
    )
    .bind(outline.id)
    .bind(outline.parent_id)
    .bind(&outline.fractional_index)
    .bind(&outline.doc)
    .bind(text)
    .bind(outline.hidden)
    .bind(outline.collapsed)
    .bind(outline.deleted)
    .bind(outline.created_at)
    .bind(outline.updated_at)
    .fetch_one(conn)
    .await
    .context("database error")
}

pub async fn paragraph<'a, E>(conn: E, paragraph: &Paragraph) -> Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO paragraphs (
                id,
                outline_id,
                fractional_index,
                doc,
                hidden,
                deleted,
                created_at,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE
            SET
                outline_id = excluded.outline_id,
                fractional_index = excluded.fractional_index,
                doc = excluded.doc,
                hidden = excluded.hidden,
                deleted = excluded.deleted,
                updated_at = excluded.updated_at
            WHERE id = excluded.id
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
    )
    .bind(paragraph.id)
    .bind(paragraph.outline_id)
    .bind(&paragraph.fractional_index)
    .bind(&paragraph.doc)
    .bind(paragraph.hidden)
    .bind(paragraph.deleted)
    .bind(paragraph.created_at)
    .bind(paragraph.updated_at)
    .fetch_one(conn)
    .await
    .context("database error")
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

    let mut query_builder = sqlx::query::<_>(&query);

    for (outline_id, path) in values {
        query_builder = query_builder.bind(outline_id);
        query_builder = query_builder.bind(path);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn app_state<'a, E>(conn: E, app_state: &AppState) -> Result<()>
where
    E: SqliteExecutor<'a>,
{
    let jsonb = serde_sqlite_jsonb::to_vec(app_state)?;

    sqlx::query!(
        r#"
            INSERT INTO kvs (id, value)
            VALUES (?, ?)
            ON CONFLICT DO UPDATE
            SET
                value = excluded.value;
        "#,
        "app_state",
        jsonb
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn workspace_state<'a, E>(conn: E, workspace_state: &WorkspaceState) -> Result<()>
where
    E: SqliteExecutor<'a>,
{
    let jsonb = serde_sqlite_jsonb::to_vec(workspace_state)?;

    sqlx::query!(
        r#"
            INSERT INTO workspaces (pot_id, value)
            VALUES (?, ?)
            ON CONFLICT DO UPDATE
            SET
                value = excluded.value;
        "#,
        workspace_state.pot.id,
        jsonb
    )
    .execute(conn)
    .await?;

    Ok(())
}
