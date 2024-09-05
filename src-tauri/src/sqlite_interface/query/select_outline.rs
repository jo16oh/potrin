use super::super::POOL;
use crate::utils::get_once_lock;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
pub struct RawOutline {
    pub id: String,
    pub parent: Option<String>,
    pub text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

struct QueryResult {
    id: Vec<u8>,
    parent_id: Option<Vec<u8>>,
    fractional_index: Option<String>,
    text: Option<String>,
    created_at: i64,
    updated_at: i64,
}

impl TryFrom<QueryResult> for RawOutline {
    type Error = anyhow::Error;

    fn try_from(value: QueryResult) -> anyhow::Result<RawOutline> {
        let id = String::from_utf8(value.id)?;
        let parent = value.parent_id.map(String::from_utf8).transpose()?;

        Ok(RawOutline {
            id,
            parent,
            text: value.text,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_outline(id: Vec<u8>) -> anyhow::Result<RawOutline> {
    let pool = get_once_lock(&POOL)?;
    sqlx::query_as!(
        QueryResult,
        r#"SELECT id, parent_id, text, fractional_index, created_at, updated_at FROM outlines WHERE id = ?;"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))?
    .try_into()
}
