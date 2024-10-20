use crate::types::{model::OutlineYUpdate, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_outline_y_updates_by_outline_id(
    pool: &SqlitePool,
    outline_id: &Base64,
) -> anyhow::Result<Vec<OutlineYUpdate>> {
    query_as!(
        OutlineYUpdate,
        r#"
            SELECT id, data
            FROM outline_y_updates
            WHERE outline_id = ?;
        "#,
        outline_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e))
}
