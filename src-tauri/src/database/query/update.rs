use crate::types::model::Pot;
use eyre::Result;
use sqlx::SqliteExecutor;

pub async fn pot<'a, E>(conn: E, pot: &Pot, updated_at: i64) -> Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            UPDATE pots
            SET 
                name = ?, 
                updated_at = ?
            WHERE id = ?;
        "#,
        pot.name,
        updated_at,
        pot.id
    )
    .execute(conn)
    .await?;

    Ok(())
}
