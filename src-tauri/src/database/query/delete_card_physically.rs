use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn delete_card_physically<'a, E>(conn: E, card_id: &Base64) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            DELETE FROM cards 
            WHERE id = ?;
        "#,
        card_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
