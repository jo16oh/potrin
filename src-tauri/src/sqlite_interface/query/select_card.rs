use super::super::POOL;
use crate::utils::get_once_lock;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{FromRow, QueryBuilder, Sqlite};

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
pub struct RawCard {
    pub id: String,
    pub outline_id: String,
    pub text: String,
    pub fractional_index: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
struct QueryResult {
    id: Vec<u8>,
    outline_id: Vec<u8>,
    fractional_index: String,
    text: String,
    created_at: i64,
    updated_at: i64,
}

impl TryFrom<QueryResult> for RawCard {
    type Error = anyhow::Error;

    fn try_from(value: QueryResult) -> anyhow::Result<RawCard> {
        let id = String::from_utf8(value.id)?;
        let outline_id = String::from_utf8(value.outline_id)?;

        Ok(RawCard {
            id,
            outline_id,
            text: value.text,
            fractional_index: value.fractional_index,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_cards(ids: Vec<Vec<u8>>) -> anyhow::Result<Vec<RawCard>> {
    let pool = get_once_lock(&POOL)?;

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT id, outline_id, fractional_index, text, created_at, updated_at FROM cards WHERE id IN (");

    let mut separated = query_builder.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    // dbg!(query_builder.sql());

    query_builder
        .build_query_as::<QueryResult>()
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
}
