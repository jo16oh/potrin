use crate::types::util::{Base64, NullableBase64, Operation, Origin};
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

#[macros::model_to_event]
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

#[macros::model_to_event]
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

#[macros::model_to_event]
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

#[macros::model_to_event]
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
