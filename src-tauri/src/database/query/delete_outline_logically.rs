use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn delete_outline_logically<'a, E>(conn: E, outline_id: &Base64) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            UPDATE outlines 
            SET is_deleted = 1
            WHERE id = ?;
        "#,
        outline_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
