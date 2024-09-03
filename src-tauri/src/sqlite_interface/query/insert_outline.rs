use super::super::POOL;
use crate::utils::get_once_lock;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Type, Clone)]
struct RawOutline {
    id: String,
    parent: Option<String>,
    text: Option<String>,
    created_at: i64,
    updated_at: i64,
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
pub async fn insert_outline(text: &str, parent: Option<Vec<u8>>) -> anyhow::Result<Vec<u8>> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().into_bytes();

    let row: RawOutline = sqlx::query_as!(
        QueryResult,
        r#"
            INSERT INTO outlines (id, parent_id, fractional_index, text, from_remote)
            VALUES (?, ?, ?, ?, ?) 
            RETURNING id, parent_id, fractional_index, text, created_at, updated_at;"#,
        id,
        parent,
        "",
        text,
        1
    )
    .fetch_one(pool)
    .await?
    .try_into()?;

    // app_handle.emit("data_change", row)?;

    Ok(id)
}
