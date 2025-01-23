use crate::types::util::UUIDv7Base64URL;
use eyre::OptionExt;
use sqlx::SqliteExecutor;

pub async fn y_doc<'a, E>(conn: E, id: UUIDv7Base64URL) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar!(
        r#"
            DELETE FROM y_docs
            WHERE id = ?
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        id,
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
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

pub mod soft {
    use super::*;

    pub async fn paragraphs<'a, E>(
        conn: E,
        paragraph_ids: &[UUIDv7Base64URL],
    ) -> eyre::Result<Vec<i64>>
    where
        E: SqliteExecutor<'a>,
    {
        let query = format!(
            r#"
                UPDATE paragraphs
                SET deleted = true
                WHERE id IN ({})
                RETURNING (
                  SELECT rowid FROM operation_logs WHERE primary_key = id
                ) AS rowid;
            "#,
            paragraph_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_scalar(&query);

        for id in paragraph_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(conn).await.map_err(|e| e.into())
    }

    pub async fn outlines<'a, E>(conn: E, outline_ids: &[UUIDv7Base64URL]) -> eyre::Result<Vec<i64>>
    where
        E: SqliteExecutor<'a>,
    {
        let query = format!(
            r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, parent_id
                    FROM outlines
                    WHERE id IN ({})
                    UNION ALL
                    SELECT
                        child.id, child.parent_id
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                )
                UPDATE outlines
                SET deleted = true
                WHERE id IN ((SELECT id FROM outline_tree))
                RETURNING (
                  SELECT rowid FROM operation_logs WHERE primary_key = id
                ) AS rowid;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_scalar(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(conn).await.map_err(|e| e.into())
    }
}
