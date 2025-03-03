use super::util::{BytesBase64URL, UUIDv7Base64URL};
use derive_more::derive::Deref;
use garde::Validate;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteValueRef, Database, Decode, FromRow, Sqlite};
use std::collections::{HashMap, VecDeque};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UUIDv7Base64URL,
    pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Validate, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Pot {
    #[garde(skip)]
    pub id: UUIDv7Base64URL,
    #[garde(length(min = 1, max = 50))]
    pub name: String,
    #[garde(skip)]
    pub owner: Option<UUIDv7Base64URL>,
    #[garde(skip)]
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
    pub collapsed: bool,
    pub deleted: bool,
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
            collapsed: false,
            deleted: false,
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
    pub collapsed: bool,
    pub deleted: bool,
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
            collapsed: false,
            deleted: false,
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
    pub deleted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
pub struct RawParagraph {
    pub id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quoted_paragraph_id: Option<UUIDv7Base64URL>,
    pub quoted_version_id: Option<UUIDv7Base64URL>,
    pub quoted_doc: Option<String>,
    pub latest_quoted_doc: Option<String>,
    pub quoted_path: Option<Path>,
    pub links: Links,
    pub hidden: bool,
    pub deleted: bool,
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
            quote: value
                .quoted_paragraph_id
                .zip(value.quoted_version_id)
                .zip(value.quoted_doc)
                .zip(value.quoted_path)
                .map(|(((id, version_id), doc), path)| Quote {
                    id,
                    version_id,
                    doc,
                    latest_doc: value.latest_quoted_doc,
                    path,
                }),
            links: value.links,
            hidden: value.hidden,
            deleted: value.deleted,
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
    pub deleted: bool,
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
            deleted: false,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct RawParagraphForIndex {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
    pub outline_id: UUIDv7Base64URL,
    pub fractional_index: String,
    pub doc: String,
    pub quoted_paragraph_id: Option<UUIDv7Base64URL>,
    pub quoted_version_id: Option<UUIDv7Base64URL>,
    pub quoted_doc: Option<String>,
    pub latest_quoted_doc: Option<String>,
    pub quoted_path: Option<Path>,
    pub path: Path,
    pub links: Links,
    pub hidden: bool,
    pub deleted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<RawParagraphForIndex> for ParagraphForIndex {
    fn from(value: RawParagraphForIndex) -> Self {
        Self {
            id: value.id,
            pot_id: value.pot_id,
            outline_id: value.outline_id,
            fractional_index: value.fractional_index,
            doc: value.doc,
            quote: value
                .quoted_paragraph_id
                .zip(value.quoted_version_id)
                .zip(value.quoted_doc)
                .zip(value.quoted_path)
                .map(|(((id, version_id), doc), path)| Quote {
                    id,
                    version_id,
                    doc,
                    latest_doc: value.latest_quoted_doc,
                    path,
                }),
            path: value.path,
            links: value.links,
            hidden: value.hidden,
            deleted: value.deleted,
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
    pub path: Path,
    pub doc: String,
    pub latest_doc: Option<String>,
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
            deleted: false,
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
    pub version_id: Option<UUIDv7Base64URL>,
    pub timestamp: i64,
}

impl YUpdate {
    pub fn new(
        y_doc_id: UUIDv7Base64URL,
        data: BytesBase64URL,
        version_id: Option<UUIDv7Base64URL>,
        timestamp: i64,
    ) -> Self {
        YUpdate {
            id: UUIDv7Base64URL::new(),
            y_doc_id,
            data,
            version_id,
            timestamp,
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

    pub fn inner(&self) -> &[Link] {
        &self.0
    }
}

impl Path {
    pub fn replace_ancestors_with(&mut self, path: &Path) {
        let len = path.0.len();
        self.0.splice(0..len, path.0.clone());
    }
}

impl From<Vec<Link>> for Path {
    fn from(value: Vec<Link>) -> Self {
        Self(value)
    }
}

impl From<VecDeque<Link>> for Path {
    fn from(value: VecDeque<Link>) -> Self {
        Self(value.into())
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

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDay {
    pub day_start: i64,
    pub paragraphs: Vec<Paragraph>,
    pub outlines: Vec<Outline>,
}

#[allow(dead_code)]
#[derive(FromRow)]
pub struct Oplog {
    pub rowid: i64,
    pub primary_key: UUIDv7Base64URL,
    pub tablename: String,
    pub operation: String,
    pub updated_at: i64,
    pub status: Option<Vec<u8>>,
}

#[allow(dead_code)]
#[derive(FromRow)]
pub struct PendingYUpdate {
    pub y_doc_id: UUIDv7Base64URL,
    pub doc_type: String,
    pub data: BytesBase64URL,
    pub timestamp: i64,
}

#[derive(FromRow)]
pub struct Descendant {
    pub id: UUIDv7Base64URL,
    pub parent_id: UUIDv7Base64URL,
    pub path: Option<Path>,
}
