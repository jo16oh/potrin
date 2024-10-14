use crate::types::model::Outline;
use sqlx::SqliteExecutor;

pub async fn update_outline<'a, E>(conn: E, outline: &Outline) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query!(
        r#"
            UPDATE outlines 
            SET 
                parent_id = ?,
                fractional_index = ?,
                text = ?
            WHERE id = ?;
        "#,
        outline.parent_id,
        outline.fractional_index,
        outline.text,
        outline.id
    )
    .execute(conn)
    .await?;

    Ok(())
}
