use super::types::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;

use tauri_specta::Event;
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct User {
    pub id: Base64,
    pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Pot {
    pub id: Base64,
    pub name: String,
    pub owner: Base64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Outline {
    pub id: Base64,
    pub parent_id: NullableBase64,
    pub fractional_index: String,
    pub text: Option<String>,
}

#[cfg(test)]
impl Outline {
    pub fn new(parent_id: Option<&Base64>) -> Self {
        Self {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: match parent_id {
                Some(id) => NullableBase64::from(id.clone()),
                None => NullableBase64::none(),
            },
            fractional_index: String::new(),
            text: Some(String::new()),
        }
    }
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OutlineYUpdate {
    pub id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: i64,
}

#[cfg(test)]
impl OutlineYUpdate {
    pub fn new() -> Self {
        Self {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            data: Base64::from(uuidv7::create_raw().to_vec()),
            created_at: chrono::Utc::now().timestamp_millis(),
            is_checkpoint: 0,
        }
    }
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct Card {
    pub id: Base64,
    pub outline_id: Base64,
    pub fractional_index: String,
    pub text: String,
}

#[cfg(test)]
impl Card {
    pub fn new(outline_id: Base64) -> Self {
        Self {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            outline_id,
            fractional_index: String::new(),
            text: String::new(),
        }
    }
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct CardYUpdate {
    pub id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: i64,
}

#[cfg(test)]
impl CardYUpdate {
    pub fn new() -> Self {
        Self {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            data: Base64::from(uuidv7::create_raw().to_vec()),
            created_at: chrono::Utc::now().timestamp_millis(),
            is_checkpoint: 0,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OplogTable {
    pub rowid: i64,
    pub primary_key: Base64,
    pub tablename: String,
    pub updated_at: i64,
    pub counter: i64,
    pub is_deleted: i64,
    pub status: NullableBase64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct UsersTable {
    pub id: Base64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct PotsTable {
    pub id: Base64,
    pub name: String,
    pub owner: Option<Base64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct SyncStatusTable {
    pub pot_id: Base64,
    pub tablename: String,
    pub shape_id: Option<String>,
    pub offset: Option<String>,
    pub last_sent_timestamp: Option<i64>,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OutlinesTable {
    pub id: Base64,
    pub author: NullableBase64,
    pub pot_id: NullableBase64,
    pub parent_id: NullableBase64,
    pub fractional_index: String,
    pub text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: i64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct OutlineYUpdatesTable {
    pub id: Base64,
    pub outline_id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: SqliteBool,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct CardsTable {
    pub id: Base64,
    pub author: NullableBase64,
    pub outline_id: Base64,
    pub fractional_index: String,
    pub text: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_deleted: i64,
}

#[macros::table_change_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct CardYUpdatesTable {
    pub id: Base64,
    pub card_id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: i64,
}
