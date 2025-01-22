use super::util::UUIDv7Base64URL;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub client_id: UUIDv7Base64URL,
    pub user: Option<UserState>,
    #[specta(type = HashMap<String, String>)]
    pub pots: HashMap<UUIDv7Base64URL, String>,
    pub setting: AppSetting,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UserState {
    pub id: UUIDv7Base64URL,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PotState {
    pub id: UUIDv7Base64URL,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    pub pot: PotState,
    pub tabs: Vec<TabState>,
    pub focus: FocusState,
    pub sidebar: SidebarState,
}

impl WorkspaceState {
    pub fn new(pot_id: UUIDv7Base64URL, pot_name: String) -> Self {
        Self {
            pot: PotState {
                id: pot_id,
                name: pot_name,
            },
            tabs: Vec::new(),
            focus: FocusState::Timeline {},
            sidebar: SidebarState {
                is_float: false,
                width: 300,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SidebarState {
    pub is_float: bool,
    pub width: u16,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            is_float: false,
            width: 300,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum FocusState {
    Timeline {},
    Search {},
    Tabs { index: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub views: Vec<ViewState>,
    pub focused_view_idx: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ViewState {
    pub id: UUIDv7Base64URL,
    pub view_type: ViewType,
    pub title: String,
    pub flex_grow: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ViewType {
    Outline,
    Relation,
    Search,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSetting {
    pub levenshtein_distance: u8,
}
