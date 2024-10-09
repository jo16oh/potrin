use crate::types::model::{CardYUpdate, OutlineYUpdate};
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqliteExecutor;

pub async fn insert_outline_y_updates<'a, E>(
    conn: E,
    outline_id: &Base64,
    y_updates: &[OutlineYUpdate],
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO outline_y_updates (id, outline_id, data, created_at, is_checkpoint)
            VALUES {};
        "#,
        y_updates
            .iter()
            .map(|_| "(?, ?, ?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for update in y_updates {
        query_builder = query_builder.bind(&update.id);
        query_builder = query_builder.bind(outline_id);
        query_builder = query_builder.bind(&update.data);
        query_builder = query_builder.bind(update.created_at);
        query_builder = query_builder.bind(update.is_checkpoint);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn insert_card_y_updates<'a, E>(
    conn: E,
    card_id: &Base64,
    y_updates: &[CardYUpdate],
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO card_y_updates (id, card_id, data, created_at, is_checkpoint)
            VALUES {};
        "#,
        y_updates
            .iter()
            .map(|_| "(?, ?, ?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for update in y_updates {
        query_builder = query_builder.bind(&update.id);
        query_builder = query_builder.bind(card_id);
        query_builder = query_builder.bind(&update.data);
        query_builder = query_builder.bind(update.created_at);
        query_builder = query_builder.bind(update.is_checkpoint);
    }

    query_builder.execute(conn).await?;

    Ok(())
}
