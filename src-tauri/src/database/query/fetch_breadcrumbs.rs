use crate::database::types::Base64String;
use crate::database::types::NullableBase64String;
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use sqlx::{prelude::FromRow, SqlitePool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Breadcrumb {
    pub id: Base64String,
    pub parent_id: NullableBase64String,
    pub text: Option<String>,
}

pub async fn fetch_breadcrumbs(
    parent_ids: Vec<&Base64String>,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<Breadcrumb>> {
    let id_string = parent_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    query_as!(
        Breadcrumb,
        r#"
            WITH RECURSIVE breadcrumbs AS (
                SELECT
                    id, parent_id, text
                FROM outlines
                WHERE id IN (?) AND is_deleted = false
                UNION ALL
                SELECT
                    child.id, child.parent_id, child.text
                FROM breadcrumbs AS parent
                JOIN outlines AS child ON parent.id = child.parent_id
                WHERE child.is_deleted = false
            )
            SELECT id, parent_id, text FROM breadcrumbs;
            "#,
        id_string
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
