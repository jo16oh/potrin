use crate::types::util::UUIDv7Base64URL;
use eyre::Context;
use sqlx::SqliteExecutor;

pub async fn pot<'a, E>(conn: E, id: UUIDv7Base64URL) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            DELETE FROM pots
            WHERE id = ?;
        "#,
        id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn y_doc<'a, E>(conn: E, id: UUIDv7Base64URL) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar::<_, i64>(
        r#"
            DELETE FROM y_docs
            WHERE id = ?
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
    )
    .bind(id)
    .fetch_one(conn)
    .await
    .context("database error")
}

pub async fn y_updates<'a, E>(conn: E, update_ids: &[UUIDv7Base64URL]) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    if update_ids.is_empty() {
        return Ok(());
    }

    let query = format!(
        r#"
            DELETE FROM y_updates
            WHERE id IN ({});
        "#,
        update_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for id in update_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn oplogs<'a, E>(conn: E, ids: &[i64]) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
            DELETE FROM operation_logs
            WHERE rowid IN ({});
        "#,
        ids.iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for id in ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.execute(conn).await?;

    Ok(())
}
