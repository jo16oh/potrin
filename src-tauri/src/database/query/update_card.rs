use crate::types::model::Card;
use sqlx::SqliteExecutor;

pub async fn update_card<'a, E>(conn: E, card: &Card) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            UPDATE cards 
            SET 
                outline_id = ?,
                fractional_index = ?,
                text = ?,
                version_id = ?
            WHERE id = ?;
        "#,
        card.outline_id,
        card.fractional_index,
        card.text,
        card.version_id,
        card.id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
