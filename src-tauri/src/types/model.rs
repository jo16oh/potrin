use crate::types::util::{Base64, NullableBase64, Operation, Origin};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;
use tauri_specta::Event;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Base64,
    pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Pot {
    pub id: Base64,
    pub name: String,
    pub owner: Base64,
}

#[macros::model_to_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct OutlineYUpdate {
    pub id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct RawCard {
    pub id: Base64,
    pub outline_id: Base64,
    pub fractional_index: String,
    pub text: String,
    pub version_id: NullableBase64,
    pub quoted_card_id: NullableBase64,
    pub quote_version_id: NullableBase64,
}

#[macros::model_to_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: Base64,
    pub outline_id: Base64,
    pub fractional_index: String,
    pub text: String,
    pub version_id: NullableBase64,
    pub quote: Option<Quote>,
}

impl From<RawCard> for Card {
    fn from(value: RawCard) -> Self {
        Self {
            id: value.id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            text: value.text,
            version_id: value.version_id,
            quote: if let Some(version_id) = value.quote_version_id.into_option() {
                Some(Quote {
                    id: value.quoted_card_id,
                    version_id,
                })
            } else {
                None
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub id: NullableBase64,
    pub version_id: Base64,
}

#[cfg(test)]
impl Card {
    pub fn new(outline_id: Base64, quote: Option<Quote>) -> Self {
        Self {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            outline_id,
            fractional_index: String::new(),
            text: String::new(),
            version_id: NullableBase64::none(),
            quote,
        }
    }
}

#[macros::model_to_event]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CardYUpdate {
    pub id: Base64,
    pub data: Base64,
    pub created_at: i64,
    pub is_checkpoint: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Breadcrumb {
    pub id: Base64,
    pub parent_id: NullableBase64,
    pub text: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LinkCount {
    pub id: Base64,
    pub back: i64,
    pub forward: i64,
}
