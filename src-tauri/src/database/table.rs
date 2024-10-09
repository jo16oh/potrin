// use super::types::*;
// use serde::{Deserialize, Serialize};
// use specta::Type;
// use sqlx::FromRow;
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct OplogTable {
//     pub rowid: i64,
//     pub primary_key: Base64,
//     pub tablename: String,
//     pub updated_at: i64,
//     pub counter: i64,
//     pub is_deleted: i64,
//     pub status: NullableBase64,
// }
//
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct UsersTable {
//     pub id: Base64,
//     pub name: String,
//     pub created_at: i64,
//     pub updated_at: i64,
// }
//
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct PotsTable {
//     pub id: Base64,
//     pub name: String,
//     pub owner: Option<Base64>,
//     pub created_at: i64,
//     pub updated_at: i64,
// }
//
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct SyncStatusTable {
//     pub pot_id: Base64,
//     pub tablename: String,
//     pub shape_id: Option<String>,
//     pub offset: Option<String>,
//     pub last_sent_timestamp: Option<i64>,
// }
//
// #[macros::table_change_event]
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct OutlinesTable {
//     pub id: Base64,
//     pub author: NullableBase64,
//     pub pot_id: NullableBase64,
//     pub parent_id: NullableBase64,
//     pub fractional_index: String,
//     pub text: Option<String>,
//     pub created_at: i64,
//     pub updated_at: i64,
//     pub is_deleted: i64,
// }
//
// #[macros::table_change_event]
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct OutlineYUpdatesTable {
//     pub id: Base64,
//     pub outline_id: Base64,
//     pub data: Base64,
//     pub created_at: i64,
//     pub is_checkpoint: SqliteBool,
// }
//
// #[macros::table_change_event]
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct CardsTable {
//     pub id: Base64,
//     pub author: NullableBase64,
//     pub outline_id: Base64,
//     pub fractional_index: String,
//     pub text: String,
//     pub created_at: i64,
//     pub updated_at: i64,
//     pub is_deleted: i64,
// }
//
// #[macros::table_change_event]
// #[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
// pub struct CardYUpdatesTable {
//     pub id: Base64,
//     pub card_id: Base64,
//     pub data: Base64,
//     pub created_at: i64,
//     pub is_checkpoint: i64,
// }
