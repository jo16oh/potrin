use crate::types::{
    model::{Pot, User, YUpdate},
    util::UUIDv7Base64URL,
};
use eyre::{OptionExt, Result};
use sqlx::SqliteExecutor;

pub mod from_local {
    use crate::types::util::BytesBase64URL;
    use crate::types::util::UUIDv7Base64URL;

    pub use super::pot;
    pub use super::user;
    use super::*;

    pub async fn version<'a, E>(
        conn: E,
        pot_id: UUIDv7Base64URL,
        version_id: UUIDv7Base64URL,
        timestamp: i64,
    ) -> eyre::Result<i64>
    where
        E: SqliteExecutor<'a>,
    {
        super::version(conn, pot_id, version_id, timestamp, false).await
    }

    pub async fn y_updates<'a, E>(
        conn: E,
        y_updates: &[YUpdate],
        version_id: Option<UUIDv7Base64URL>,
        timestamp: i64,
    ) -> eyre::Result<Vec<i64>>
    where
        E: SqliteExecutor<'a>,
    {
        super::y_updates(conn, y_updates, version_id, timestamp, false).await
    }

    pub async fn pending_y_update<'a, E>(
        conn: E,
        y_doc_id: UUIDv7Base64URL,
        y_update: &BytesBase64URL,
    ) -> eyre::Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        sqlx::query!(
            r#"
                INSERT INTO pending_y_updates (y_doc_id, data)
                VALUES (?, ?);
            "#,
            y_doc_id,
            y_update
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
        timestamp: i64,
    ) -> eyre::Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        super::y_doc(conn, id, pot_id, user_id, doc_type, timestamp, false).await
    }
}

pub mod from_remote {}

pub async fn user<'a, E>(conn: E, user: &User, timestamp: i64) -> Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO users (id, name, created_at, updated_at)
            VALUES (?, ?, ?, ?);
        "#,
        user.id,
        user.name,
        timestamp,
        timestamp
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn pot<'a, E>(conn: E, pot: &Pot, timestamp: i64) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_scalar!(
        r#"
            INSERT OR IGNORE INTO pots (id, name, owner, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        pot.id,
        pot.name,
        pot.owner,
        timestamp,
        timestamp
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
}

async fn y_doc<'a, E>(
    conn: E,
    id: UUIDv7Base64URL,
    pot_id: UUIDv7Base64URL,
    user_id: Option<UUIDv7Base64URL>,
    doc_type: &str,
    timestamp: i64,
    from_remote: bool,
) -> eyre::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let from_remote = if from_remote { 1 } else { 0 };

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO y_docs (id, pot_id, author, type, created_at, from_remote)
            VALUES (?, ?, ?, ?, ?, ?);
        "#,
        id,
        pot_id,
        user_id,
        doc_type,
        timestamp,
        from_remote
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn y_updates<'a, E>(
    conn: E,
    y_updates: &[YUpdate],
    version_id: Option<UUIDv7Base64URL>,
    timestamp: i64,
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
            INSERT INTO y_updates (id, y_doc_id, data, version_id, created_at, from_remote)
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
        query_builder = query_builder.bind(version_id);
        query_builder = query_builder.bind(timestamp);
        query_builder = query_builder.bind(from_remote);
    }

    query_builder.fetch_all(conn).await.map_err(|e| e.into())
}

async fn version<'a, E>(
    conn: E,
    pot_id: UUIDv7Base64URL,
    version_id: UUIDv7Base64URL,
    timestamp: i64,
    from_remote: bool,
) -> eyre::Result<i64>
where
    E: SqliteExecutor<'a>,
{
    let from_remote = if from_remote { 1 } else { 0 };

    sqlx::query_scalar!(
        r#"
            INSERT OR IGNORE INTO versions (id, pot_id, created_at, from_remote)
            VALUES (?, ?, ?, ?)
            RETURNING (
              SELECT rowid FROM operation_logs WHERE primary_key = id
            ) AS rowid;
        "#,
        version_id,
        pot_id,
        timestamp,
        from_remote
    )
    .fetch_one(conn)
    .await?
    .ok_or_eyre("failed to insert into oplog")
}
