use crate::types::model::User;
use sqlx::SqliteExecutor;

pub async fn insert_user<'a, E>(conn: E, user: &User) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            INSERT INTO users (id, name)
            VALUES (?, ?);
        "#,
        user.id,
        user.name,
    )
    .execute(conn)
    .await?;

    Ok(())
}
