use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn delete_quote<'a, E>(conn: E, card_id: &Base64) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            DELETE FROM quotes 
            WHERE card_id = ?;
        "#,
        card_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
