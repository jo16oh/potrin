use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;
use tauri_specta::Event;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct OplogTable {
    pub rowid: i64,
    pub primary_key: Vec<u8>,
    pub tablename: String,
    pub updated_at: i64,
    pub counter: i64,
    pub is_deleted: i64,
    pub status: Option<Vec<u8>>,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct UsersTable {
    pub id: Vec<u8>,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct PotsTable {
    pub id: Vec<u8>,
    pub name: String,
    pub owner: Option<i64>,
    pub sync: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct SyncStatusTable {
    pub pot_id: Vec<u8>,
    pub tablename: String,
    pub shape_id: Option<String>,
    pub offset: Option<String>,
    pub last_sent_timestamp: Option<i64>,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct OutlinesTable {
    pub id: Vec<u8>,
    pub author: Vec<u8>,
    pub pot_id: Option<Vec<u8>>,
    pub parent_id: Vec<u8>,
    pub fractional_index: String,
    pub text: String,
    pub last_materialized_hash: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: bool,
    pub from_remote: bool,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct OutlineYUpdatesTable {
    pub id: Vec<u8>,
    pub outline_id: Vec<u8>,
    pub data: Vec<u8>,
    pub updated_at: i64,
    pub is_checkpoint: bool,
    pub from_remote: bool,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type)]
pub struct CardsTable {
    pub id: Vec<u8>,
    pub author: Option<Vec<u8>>,
    pub outline_id: Vec<u8>,
    pub fractional_index: String,
    pub text: String,
    pub last_materialized_hash: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: i64,
    pub from_remote: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct CardsTableChangeEvent {
    pub operation: Operation,
    pub rows_changed: Vec<CardsTable>,
}

impl CardsTableChangeEvent {
    pub fn new(operation: Operation, rows: &[CardsTable]) -> Self {
        Self {
            operation,
            rows_changed: rows.to_vec(),
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, Type, Event)]
pub struct CardYUpdatesTable {
    pub id: Vec<u8>,
    pub card_id: Vec<u8>,
    pub data: Vec<u8>,
    pub updated_at: i64,
    pub is_checkpoint: bool,
    pub from_remote: bool,
}
