use super::types::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;
use tauri_specta::Event;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OplogTable {
    pub rowid: i64,
    pub primary_key: Base64String,
    pub tablename: String,
    pub updated_at: i64,
    pub counter: i64,
    pub is_deleted: i64,
    pub status: NullableBase64String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct UsersTable {
    pub id: Base64String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct PotsTable {
    pub id: Base64String,
    pub name: String,
    pub owner: Option<i64>,
    pub sync: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct SyncStatusTable {
    pub pot_id: Base64String,
    pub tablename: String,
    pub shape_id: Option<String>,
    pub offset: Option<String>,
    pub last_sent_timestamp: Option<i64>,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OutlinesTable {
    pub id: Base64String,
    pub author: NullableBase64String,
    pub pot_id: NullableBase64String,
    pub parent_id: NullableBase64String,
    pub fractional_index: String,
    pub text: Option<String>,
    pub last_materialized_hash: NullableBase64String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: i64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OutlineYUpdatesTable {
    pub id: Base64String,
    pub outline_id: Base64String,
    pub data: Base64String,
    pub updated_at: i64,
    pub is_checkpoint: i64,
    pub from_remote: i64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct CardsTable {
    pub id: Base64String,
    pub author: NullableBase64String,
    pub outline_id: Base64String,
    pub fractional_index: String,
    pub text: String,
    pub last_materialized_hash: NullableBase64String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: i64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct CardYUpdatesTable {
    pub id: Base64String,
    pub card_id: Base64String,
    pub data: Base64String,
    pub updated_at: i64,
    pub is_checkpoint: i64,
    pub from_remote: i64,
}
