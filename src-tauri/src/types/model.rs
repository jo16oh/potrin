use super::util::{BytesBase64URL, UUIDv7Base64URL};
use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteValueRef, Database, Decode, FromRow, Sqlite};
use std::collections::HashMap;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UUIDv7Base64URL,
    pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Pot {
    pub id: UUIDv7Base64URL,
    pub name: String,
    pub owner: Option<UUIDv7Base64URL>,
    pub created_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    pub id: UUIDv7Base64URL,
    pub parent_id: Option<UUIDv7Base64URL>,
    pub fractional_index: String,
    pub doc: String,
    #[sqlx(default)]
    pub text: String,
    #[sqlx(default)]
    pub path: Option<Path>,
    pub links: Links,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Outline {
    #[cfg(test)]
    pub fn new(parent_id: Option<UUIDv7Base64URL>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: UUIDv7Base64URL::new(),
            parent_id,
            fractional_index: String::new(),
            doc: String::new(),
            text: String::new(),
            path: None,
            links: Links(HashMap::new()),
            hidden: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OutlineForIndex {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
    pub parent_id: Option<UUIDv7Base64URL>,
    pub fractional_index: String,
    pub doc: String,
    #[sqlx(default)]
    pub text: String,
    pub path: Path,
    pub links: Links,
    pub hidden: bool,
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
            path: Some(value.path),
            links: value.links,
            hidden: value.hidden,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    pub id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quote: Option<Quote>,
    pub links: Links,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
pub struct RawParagraph {
    pub id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quoted_id: Option<UUIDv7Base64URL>,
    pub quote_version_id: Option<UUIDv7Base64URL>,
    pub latest_quote_version_id: Option<UUIDv7Base64URL>,
    pub links: Links,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<RawParagraph> for Paragraph {
    fn from(value: RawParagraph) -> Self {
        Self {
            id: value.id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: if let (Some(quoted_id), Some(version_id)) =
                (value.quoted_id, value.quote_version_id)
            {
                Some(Quote {
                    id: quoted_id,
                    version_id,
                    doc: String::new(),
                    is_latest: value.quote_version_id == value.latest_quote_version_id,
                })
            } else {
                None
            },
            links: value.links,
            hidden: value.hidden,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphForIndex {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quote: Option<Quote>,
    pub path: Path,
    pub links: Links,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<ParagraphForIndex> for Paragraph {
    fn from(value: ParagraphForIndex) -> Self {
        Self {
            id: value.id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: value.quote,
            links: value.links,
            hidden: value.hidden,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct RawparagraphForIndex {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quoted_id: Option<UUIDv7Base64URL>,
    pub quote_version_id: Option<UUIDv7Base64URL>,
    pub latest_quote_version_id: Option<UUIDv7Base64URL>,
    pub path: Path,
    pub links: Links,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<RawparagraphForIndex> for ParagraphForIndex {
    fn from(value: RawparagraphForIndex) -> Self {
        Self {
            id: value.id,
            pot_id: value.pot_id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: if let (Some(quoted_id), Some(version_id)) =
                (value.quoted_id, value.quote_version_id)
            {
                Some(Quote {
                    id: quoted_id,
                    version_id,
                    doc: String::new(),
                    is_latest: value.quote_version_id == value.latest_quote_version_id,
                })
            } else {
                None
            },
            path: value.path,
            links: value.links,
            hidden: value.hidden,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub id: UUIDv7Base64URL,
    pub version_id: UUIDv7Base64URL,
    pub doc: String,
    pub is_latest: bool,
}

impl Paragraph {
    #[cfg(test)]
    pub fn new(outline_id: UUIDv7Base64URL, quote: Option<Quote>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: UUIDv7Base64URL::new(),
            outline_id,
            fractional_index: String::new(),
            doc: String::new(),
            quote,
            links: Links(HashMap::new()),
            hidden: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct YUpdate {
    pub id: UUIDv7Base64URL,
    pub y_doc_id: UUIDv7Base64URL,
    pub data: BytesBase64URL,
}

impl YUpdate {
    pub fn new(y_doc_id: UUIDv7Base64URL, data: BytesBase64URL) -> Self {
        YUpdate {
            id: UUIDv7Base64URL::new(),
            y_doc_id,
            data,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Ancestor {
    pub id: UUIDv7Base64URL,
    pub parent_id: Option<UUIDv7Base64URL>,
    pub text: String,
    pub hidden: bool,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LinkCount {
    pub id: UUIDv7Base64URL,
    pub back: i64,
    pub forward: i64,
}

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: UUIDv7Base64URL,
    pub text: String,
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Deref, Clone, Debug, specta::Type)]
#[serde(transparent)]
pub struct Path(Vec<Link>);

#[cfg(test)]
impl Path {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl sqlx::Type<Sqlite> for Path {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for Path {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;
        Ok(Path(serde_sqlite_jsonb::from_slice(bytes)?))
    }
}

#[derive(FromRow, Serialize, Deserialize, Deref, Clone, Debug, specta::Type)]
pub struct Links(#[specta(type = HashMap<String, Path>)] HashMap<UUIDv7Base64URL, Path>);

#[cfg(test)]
impl Links {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl From<Vec<Path>> for Links {
    fn from(value: Vec<Path>) -> Self {
        let res = value
            .into_iter()
            .filter_map(|path| {
                #[allow(clippy::manual_map)]
                if let Some(link) = path.last() {
                    Some((link.id, path))
                } else {
                    None
                }
            })
            .collect::<HashMap<UUIDv7Base64URL, Path>>();

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

        let links: Vec<Path> = serde_sqlite_jsonb::from_slice(bytes)?;

        Ok(Links::from(links))
    }
}

#[derive(FromRow)]
pub struct Oplog {
    pub rowid: i64,
    pub primary_key: UUIDv7Base64URL,
    pub tablename: String,
    pub operation: String,
    pub updated_at: i64,
    pub status: Option<Vec<u8>>,
}
