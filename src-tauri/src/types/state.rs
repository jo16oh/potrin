use super::model::Pot;
use crate::types::{setting::AppSetting, util::UUIDv7Base64URL};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, Default)]
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
pub struct WorkspaceState {
    pub pot: Pot,
    pub tabs: Vec<TabState>,
    pub focused_tab_id: Option<String>,
    pub sidebar: SidebarState,
}

impl WorkspaceState {
    pub fn new(pot: &Pot) -> Self {
        Self {
            pot: pot.clone(),
            tabs: Vec::new(),
            focused_tab_id: None,
            sidebar: SidebarState {
                is_float: false,
                width: 20.0,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SidebarState {
    pub is_float: bool,
    pub width: f64,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            is_float: false,
            width: 20.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub id: String,
    pub views: Vec<ViewState>,
    pub focused_view_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ViewState {
    #[serde(rename_all = "camelCase")]
    Cards {
        id: String,
        outline_id: Option<UUIDv7Base64URL>,
        title: String,
        flex_grow: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
        pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Outline {
        id: String,
        outline_id: Option<UUIDv7Base64URL>,
        title: String,
        flex_grow: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
        pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Document {
        id: String,
        outline_id: Option<UUIDv7Base64URL>,
        title: String,
        flex_grow: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
        pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Timeline {
        id: String,
        flex_grow: f64,
        scroll_position: u32,
        pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Relation {
        id: String,
        outline_id: UUIDv7Base64URL,
        title: String,
        direction: RelationDirection,
        flex_grow: f64,
        scroll_position: u32,
        pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Search {
        id: String,
        query: String,
        scope: Option<UUIDv7Base64URL>,
        flex_grow: f64,
        scroll_position: u32,
        pinned: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FocusPosition {
    id: Option<String>,
    position: EditorFocusPosition,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum EditorFocusPosition {
    Number(f64),
    Boolean(bool),
    String(PositionString),
    Null,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum PositionString {
    All,
    Start,
    End,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RelationDirection {
    Back,
    Forward,
}
