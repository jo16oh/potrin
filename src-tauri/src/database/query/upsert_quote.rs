use crate::types::model::Quote;
use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn upsert_quote<'a, E>(conn: E, card_id: &Base64, quote: &Quote) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            INSERT INTO quotes (card_id, quoted_card_id, version_id)
            VALUES (?, ?, ?)
            ON CONFLICT 
            DO UPDATE
            SET 
                quoted_card_id = excluded.quoted_card_id,
                version_id = excluded.version_id;
        "#,
        card_id,
        quote.id,
        quote.version_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
