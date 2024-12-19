use super::util::{BytesBase64, UUIDv7Base64};
use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteValueRef, Database, Decode, FromRow, Sqlite};
use std::collections::HashMap;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UUIDv7Base64,
    pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Pot {
    pub id: UUIDv7Base64,
    pub name: String,
    pub owner: Option<UUIDv7Base64>,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    pub id: UUIDv7Base64,
    pub parent_id: Option<UUIDv7Base64>,
    pub fractional_index: String,
    pub doc: String,
    #[sqlx(default)]
    pub text: String,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Outline {
    #[cfg(test)]
    pub fn new(parent_id: Option<UUIDv7Base64>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: UUIDv7Base64::new(),
            parent_id,
            fractional_index: String::new(),
            doc: String::new(),
            text: String::new(),
            links: Links(HashMap::new()),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OutlineForIndex {
    pub id: UUIDv7Base64,
    pub pot_id: UUIDv7Base64,
    pub parent_id: Option<UUIDv7Base64>,
    pub fractional_index: String,
    pub doc: String,
    #[sqlx(default)]
    pub text: String,
    pub breadcrumbs: Breadcrumbs,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<OutlineForIndex> for Outline {
    fn from(value: OutlineForIndex) -> Self {
        Outline {
            id: value.id,
            parent_id: value.parent_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            text: value.text,
            links: value.links,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: UUIDv7Base64,
    pub outline_id: UUIDv7Base64,
    pub fractional_index: String,
    pub doc: String,
    pub quote: Option<Quote>,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
pub struct RawCard {
    pub id: UUIDv7Base64,
    pub outline_id: UUIDv7Base64,
    pub fractional_index: String,
    pub doc: String,
    pub quote_id: Option<UUIDv7Base64>,
    pub quote_version_id: Option<UUIDv7Base64>,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<RawCard> for Card {
    fn from(value: RawCard) -> Self {
        Self {
            id: value.id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: if let (Some(quote_id), Some(version_id)) =
                (value.quote_id, value.quote_version_id)
            {
                Some(Quote {
                    id: quote_id,
                    version_id,
                })
            } else {
                None
            },
            links: value.links,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CardForIndex {
    pub id: UUIDv7Base64,
    pub pot_id: UUIDv7Base64,
    pub outline_id: UUIDv7Base64,
    pub fractional_index: String,
    pub doc: String,
    pub quote: Option<Quote>,
    pub breadcrumbs: Breadcrumbs,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<CardForIndex> for Card {
    fn from(value: CardForIndex) -> Self {
        Self {
            id: value.id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: value.quote,
            links: value.links,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct RawCardForIndex {
    pub id: UUIDv7Base64,
    pub pot_id: UUIDv7Base64,
    pub outline_id: UUIDv7Base64,
    pub fractional_index: String,
    pub doc: String,
    pub quote_id: Option<UUIDv7Base64>,
    pub quote_version_id: Option<UUIDv7Base64>,
    pub breadcrumbs: Breadcrumbs,
    pub links: Links,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<RawCardForIndex> for CardForIndex {
    fn from(value: RawCardForIndex) -> Self {
        Self {
            id: value.id,
            pot_id: value.pot_id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: if let (Some(quote_id), Some(version_id)) =
                (value.quote_id, value.quote_version_id)
            {
                Some(Quote {
                    id: quote_id,
                    version_id,
                })
            } else {
                None
            },
            breadcrumbs: value.breadcrumbs,
            links: value.links,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub id: UUIDv7Base64,
    pub version_id: UUIDv7Base64,
}

impl Card {
    #[cfg(test)]
    pub fn new(outline_id: UUIDv7Base64, quote: Option<Quote>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: UUIDv7Base64::new(),
            outline_id,
            fractional_index: String::new(),
            doc: String::new(),
            quote,
            links: Links(HashMap::new()),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct YUpdate {
    pub id: UUIDv7Base64,
    pub y_doc_id: UUIDv7Base64,
    pub data: BytesBase64,
}

impl YUpdate {
    pub fn new(y_doc_id: UUIDv7Base64, data: BytesBase64) -> Self {
        YUpdate {
            id: UUIDv7Base64::new(),
            y_doc_id,
            data,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Ancestor {
    pub id: UUIDv7Base64,
    pub parent_id: Option<UUIDv7Base64>,
    pub doc: String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LinkCount {
    pub id: UUIDv7Base64,
    pub back: i64,
    pub forward: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: UUIDv7Base64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Deref, Clone, Debug, specta::Type)]
#[serde(transparent)]
pub struct Breadcrumbs(Vec<Link>);

#[cfg(test)]
impl Breadcrumbs {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl sqlx::Type<Sqlite> for Breadcrumbs {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for Breadcrumbs {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;
        Ok(Breadcrumbs(serde_sqlite_jsonb::from_slice(bytes)?))
    }
}

#[derive(FromRow, Serialize, Deserialize, Deref, Clone, Debug, specta::Type)]
pub struct Links(#[specta(type = HashMap<String, Breadcrumbs>)] HashMap<UUIDv7Base64, Breadcrumbs>);

#[cfg(test)]
impl Links {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl From<Vec<Breadcrumbs>> for Links {
    fn from(value: Vec<Breadcrumbs>) -> Self {
        let res = value
            .into_iter()
            .filter_map(|breadcrumbs| {
                #[allow(clippy::manual_map)]
                if let Some(link) = breadcrumbs.last() {
                    Some((link.id, breadcrumbs))
                } else {
                    None
                }
            })
            .collect::<HashMap<UUIDv7Base64, Breadcrumbs>>();

        Self(res)
    }
}

impl sqlx::Type<Sqlite> for Links {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for Links {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;

        let links: Vec<Breadcrumbs> = serde_sqlite_jsonb::from_slice(bytes)?;

        Ok(Links::from(links))
    }
}

#[derive(FromRow)]
pub struct Oplog {
    pub rowid: i64,
    pub primary_key: UUIDv7Base64,
    pub tablename: String,
    pub operation: String,
    pub updated_at: i64,
    pub status: Option<Vec<u8>>,
}
