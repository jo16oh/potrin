use crate::types::model::Pot;
use sqlx::SqliteExecutor;

pub async fn insert_pot<'a, E>(conn: E, pot: &Pot) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            INSERT INTO pots (id, name, owner)
            VALUES (?, ?, ?);
        "#,
        pot.id,
        pot.name,
        pot.owner
    )
    .execute(conn)
    .await?;

    Ok(())
}
