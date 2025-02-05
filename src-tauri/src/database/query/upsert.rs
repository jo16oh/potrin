use crate::types::{
    model::{Outline, Paragraph},
    state::{AppState, WorkspaceState},
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
                hidden = excluded.hidden,
                collapsed = excluded.collapsed,
                deleted = excluded.deleted,
                updated_at = excluded.updated_at
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
        outline.hidden,
        outline.collapsed,
        outline.deleted,
        outline.created_at,
        outline.updated_at,
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
}

pub async fn paragraph<'a, E>(conn: E, paragraph: &Paragraph) -> Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar!(
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
        paragraph.id,
        paragraph.outline_id,
        paragraph.fractional_index,
        paragraph.doc,
        paragraph.hidden,
        paragraph.deleted,
        paragraph.created_at,
        paragraph.updated_at,
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
