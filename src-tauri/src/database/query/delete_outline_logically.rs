use crate::types::util::Base64;
use sqlx::{Sqlite, Transaction};

pub async fn delete_outline_logically<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    outline_id: &Base64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            CREATE TEMPORARY TABLE tree AS 
                WITH RECURSIVE tree AS (
                    SELECT id 
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT child.id 
                    FROM tree AS parent
                    INNER JOIN outlines AS child ON child.parent_id = parent.id
                    WHERE is_deleted = false
                )
                SELECT id FROM tree;
        "#,
        outline_id,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
            UPDATE outlines
            SET 
                is_deleted = true 
            WHERE id IN (
                SELECT id 
                FROM tree
            );
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
            UPDATE cards
            SET 
                is_deleted = true 
            WHERE outline_id IN (
                SELECT id 
                FROM tree
            );
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
            DROP TABLE temp.tree;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
