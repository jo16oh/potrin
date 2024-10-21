use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow)]
pub struct RawOutlineYUpdate {
    pub id: Base64,
    pub outline_id: Base64,
    pub data: Vec<u8>,
}

#[derive(FromRow)]
pub struct RawCardYUpdate {
    pub id: Base64,
    pub card_id: Base64,
    pub data: Vec<u8>,
}

pub async fn fetch_unversioned_outline_y_updates(
    pool: &SqlitePool,
    outline_ids: Vec<&Base64>,
) -> anyhow::Result<Vec<RawOutlineYUpdate>> {
    let query = format!(
        r#"
            SELECT id, outline_id, data
            FROM outline_y_updates
            LEFT JOIN outline_y_updates_versions ON outline_y_updates.id = outline_y_updates_versions.y_update_id
            WHERE outline_id IN ({}) AND outline_y_updates_versions.y_update_id IS NULL;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawOutlineYUpdate>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}

pub async fn fetch_unversioned_card_y_updates(
    pool: &SqlitePool,
    card_ids: Vec<&Base64>,
) -> anyhow::Result<Vec<RawCardYUpdate>> {
    let query = format!(
        r#"
            SELECT id, card_id, data
            FROM card_y_updates
            LEFT JOIN card_y_updates_versions ON card_y_updates.id = card_y_updates_versions.y_update_id
            WHERE card_id IN ({}) AND card_y_updates_versions.y_update_id IS NULL;
        "#,
        card_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, RawCardYUpdate>(&query);

    for id in card_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
