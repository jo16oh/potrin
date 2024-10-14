use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn delete_card_logically<'a, E>(conn: E, card_id: &Base64) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            UPDATE cards 
            SET is_deleted = 1
            WHERE id = ?;
        "#,
        card_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
