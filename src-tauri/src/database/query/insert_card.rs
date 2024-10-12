use crate::types::model::Card;
use sqlx::SqliteExecutor;

pub async fn insert_card<'a, E>(conn: E, card: &Card) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_as!(
        Card,
        r#"
            INSERT INTO cards (id, outline_id, fractional_index, text)
            VALUES (?, ?, ?, ?);
        "#,
        card.id,
        card.outline_id,
        card.fractional_index,
        card.text
    )
    .execute(conn)
    .await?;

    Ok(())
}
