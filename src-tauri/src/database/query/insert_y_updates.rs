use crate::database::{
    table::{CardYUpdate, OutlineYUpdate},
    types::Base64String,
};
use anyhow::anyhow;
use sqlx::{Sqlite, Transaction};

pub async fn insert_outline_y_updates(
    tx: &mut Transaction<'_, Sqlite>,
    outline_id: &Base64String,
    y_updates: Vec<OutlineYUpdate>,
) -> anyhow::Result<()> {
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO outline_y_updates (id, outline_id, data, updated_at, is_checkpoint)
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
        query_builder = query_builder.bind(update.id);
        query_builder = query_builder.bind(outline_id);
        query_builder = query_builder.bind(update.data);
        query_builder = query_builder.bind(update.updated_at);
        query_builder = query_builder.bind(update.is_checkpoint);
    }

    query_builder.execute(&mut **tx).await?;

    Ok(())
}

pub async fn insert_card_y_updates(
    tx: &mut Transaction<'_, Sqlite>,
    card_id: &Base64String,
    y_updates: Vec<CardYUpdate>,
) -> anyhow::Result<()> {
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO card_y_updates (id, card_id, data, updated_at, is_checkpoint)
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
        query_builder = query_builder.bind(update.id);
        query_builder = query_builder.bind(card_id);
        query_builder = query_builder.bind(update.data);
        query_builder = query_builder.bind(update.updated_at);
        query_builder = query_builder.bind(update.is_checkpoint);
    }

    query_builder.execute(&mut **tx).await?;

    Ok(())
}
