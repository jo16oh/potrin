use crate::types::model::Outline;
use crate::types::util::Base64;
use sqlx::SqliteExecutor;

pub async fn insert_outline<'a, E>(
    conn: E,
    outline: &Outline,
    pot_id: &Base64,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_as!(
        Outline,
        r#"
            INSERT INTO outlines (id, pot_id, parent_id, fractional_index, text)
            VALUES (?, ?, ?, ?, ?);
        "#,
        outline.id,
        pot_id,
        outline.parent_id,
        outline.fractional_index,
        outline.text
    )
    .execute(conn)
    .await?;

    Ok(())
}
