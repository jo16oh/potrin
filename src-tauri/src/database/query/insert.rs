use crate::types::{
    model::{Pot, YUpdate},
    util::UUIDv7Base64URL,
};
use eyre::Context;
use sqlx::SqliteExecutor;

pub mod from_local {
    use chrono::Utc;

    use crate::types::util::BytesBase64URL;
    use crate::types::util::UUIDv7Base64URL;

    pub use super::pot;
    use super::*;

    pub async fn version<'a, E>(
        conn: E,
        pot_id: UUIDv7Base64URL,
        version_id: UUIDv7Base64URL,
    ) -> eyre::Result<i64>
    where
        E: SqliteExecutor<'a>,
    {
        super::version(conn, pot_id, version_id, false).await
    }

    pub async fn y_updates<'a, E>(conn: E, y_updates: &[YUpdate]) -> eyre::Result<Vec<i64>>
    where
        E: SqliteExecutor<'a>,
    {
        super::y_updates(conn, y_updates, false).await
    }

    pub async fn pending_y_update<'a, E>(
        conn: E,
        y_doc_id: UUIDv7Base64URL,
        y_update: &BytesBase64URL,
    ) -> eyre::Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        let now = Utc::now().timestamp_millis();

        sqlx::query!(
            r#"
                INSERT INTO pending_y_updates (y_doc_id, data, timestamp)
                VALUES (?, ?, ?);
            "#,
            y_doc_id,
            y_update,
            now
        )
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn y_doc<'a, E>(
        conn: E,
        doc_type: &str,
        id: UUIDv7Base64URL,
        pot_id: UUIDv7Base64URL,
        user_id: Option<UUIDv7Base64URL>,
    ) -> eyre::Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        super::y_doc(conn, id, pot_id, user_id, doc_type, false).await
    }
}

pub mod from_remote {}

// pub async fn user<'a, E>(conn: E, user: &User, timestamp: i64) -> Result<()>
// where
//     E: SqliteExecutor<'a>,
// {
//     sqlx::query!(
//         r#"
//             INSERT OR IGNORE INTO users (id, name, created_at, updated_at)
//             VALUES (?, ?, ?, ?);
//         "#,
//         user.id,
//         user.name,
//         timestamp,
//         timestamp
//     )
//     .execute(conn)
//     .await?;
//
//     Ok(())
// }

pub async fn pot<'a, E>(conn: E, pot: &Pot, timestamp: i64) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar::<_, i64>(
        r#"
            INSERT OR IGNORE INTO pots (id, name, owner, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
    )
    .bind(pot.id)
    .bind(&pot.name)
    .bind(pot.owner)
    .bind(timestamp)
    .bind(timestamp)
    .fetch_one(conn)
    .await
    .context("database error")
}

async fn y_doc<'a, E>(
    conn: E,
    id: UUIDv7Base64URL,
    pot_id: UUIDv7Base64URL,
    user_id: Option<UUIDv7Base64URL>,
    doc_type: &str,
    from_remote: bool,
) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let from_remote = if from_remote { 1 } else { 0 };

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO y_docs (id, pot_id, author, type, from_remote)
            VALUES (?, ?, ?, ?, ?);
        "#,
        id,
        pot_id,
        user_id,
        doc_type,
        from_remote
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn y_updates<'a, E>(
    conn: E,
    y_updates: &[YUpdate],
    from_remote: bool,
) -> eyre::Result<Vec<i64>>
where
    E: SqliteExecutor<'a>,
{
    if y_updates.is_empty() {
        return Ok(Vec::new());
    }

    let from_remote = if from_remote { 1 } else { 0 };

    let query = format!(
        r#"
            INSERT OR IGNORE INTO y_updates (id, y_doc_id, data, version_id, timestamp, from_remote)
            VALUES {}
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        y_updates
            .iter()
            .map(|_| "(?, ?, ?, ?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_scalar::<_, i64>(&query);

    for update in y_updates {
        query_builder = query_builder.bind(update.id);
        query_builder = query_builder.bind(update.y_doc_id);
        query_builder = query_builder.bind(&update.data);
        query_builder = query_builder.bind(update.version_id);
        query_builder = query_builder.bind(update.timestamp);
        query_builder = query_builder.bind(from_remote);
    }

    query_builder
        .fetch_all(conn)
        .await
        .context("database error")
}

async fn version<'a, E>(
    conn: E,
    pot_id: UUIDv7Base64URL,
    version_id: UUIDv7Base64URL,
    from_remote: bool,
) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    let from_remote = if from_remote { 1 } else { 0 };

    sqlx::query_scalar::<_, i64>(
        r#"
            INSERT OR IGNORE INTO versions (id, pot_id, from_remote)
            VALUES (?, ?, ?)
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
    )
    .bind(version_id)
    .bind(pot_id)
    .bind(from_remote)
    .fetch_one(conn)
    .await
    .context("database error")
}

pub async fn y_doc_trees_of_version<'a, E>(
    conn: E,
    y_doc_ids: &[UUIDv7Base64URL],
    version_id: UUIDv7Base64URL,
) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
            WITH RECURSIVE docs AS (
                SELECT id, type
                FROM y_docs
                WHERE id IN ({})
            ),
            updated AS (
                SELECT outlines.id, outlines.parent_id
                FROM outlines
                INNER JOIN docs ON outlines.id = docs.id AND type = 'outline'
                UNION
                SELECT outlines.id, outlines.parent_id
                FROM paragraphs
                INNER JOIN docs ON paragraphs.id = docs.id AND type = 'paragraph'
                INNER JOIN outlines ON outlines.id = paragraphs.outline_id
            ),
            ancestors AS (
                SELECT id, parent_id
                FROM outlines
                WHERE id IN ((
                    SELECT parent_id 
                    FROM updated
                ))
                UNION
                SELECT parent.id, parent.parent_id
                FROM outlines AS parent
                INNER JOIN ancestors AS child ON child.parent_id = parent.id
            ),
            descendants AS (
                SELECT id, parent_id
                FROM outlines
                WHERE parent_id IN ((
                    SELECT id 
                    FROM updated
                ))
                UNION ALL
                SELECT child.id, child.parent_id
                FROM outlines AS child
                INNER JOIN descendants AS parent ON child.parent_id = parent.id
            )
            INSERT OR IGNORE INTO y_doc_trees_as_of_version
            SELECT ? AS version_id, id, parent_id
            FROM updated
            UNION ALL
            SELECT ? AS version_id, id, parent_id
            FROM ancestors
            UNION  ALL
            SELECT ? AS version_id, id, parent_id
            FROM descendants
            UNION ALL
            SELECT ? AS version_id, paragraphs.id, outline_id AS parent_id
            FROM paragraphs
            INNER JOIN updated ON updated.id = paragraphs.outline_id
            INNER JOIN ancestors ON ancestors.id = paragraphs.outline_id
            INNER JOIN descendants ON descendants.id = paragraphs.outline_id;
        "#,
        y_doc_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query::<_>(&query);

    for id in y_doc_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder = query_builder.bind(version_id);
    query_builder = query_builder.bind(version_id);
    query_builder = query_builder.bind(version_id);
    query_builder = query_builder.bind(version_id);

    query_builder.execute(conn).await?;

    Ok(())
}
