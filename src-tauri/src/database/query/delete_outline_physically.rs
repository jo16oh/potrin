use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn delete_outline_physically<'a, E>(conn: E, outline_id: &Base64) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            DELETE FROM outlines 
            WHERE id = ?;
        "#,
        outline_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
