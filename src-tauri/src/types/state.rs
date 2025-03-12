use crate::{
    search_engine::OrderBy,
    types::{model::Pot, setting::AppSetting, util::UUIDv7Base64URL},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    pub pinned_tabs: Vec<PinnedTabState>,
    pub tabs: Vec<TabState>,
    pub focused_tab_id: Option<String>,
    pub sidebar: SidebarState,
}

impl WorkspaceState {
    pub fn new(pot: &Pot) -> Self {
        let timeline_view_id = Uuid::new_v4().to_string();
        let mut pinned_view_ids_timeline = HashMap::<String, ()>::new();
        pinned_view_ids_timeline.insert(timeline_view_id.clone(), ());
        let mut pinned_view_ids_search = HashMap::<String, ()>::new();
        pinned_view_ids_search.insert(timeline_view_id.clone(), ());

        Self {
            pot: pot.clone(),
            pinned_tabs: vec![
                PinnedTabState {
                    id: Uuid::new_v4().to_string(),
                    views: vec![ViewState::Timeline {
                        id: timeline_view_id.clone(),
                        view_width_ratio: 1.0,
                        position: None,
                    }],
                    focused_view_id: None,
                    pinned_view_ids: pinned_view_ids_timeline,
                },
                PinnedTabState {
                    id: Uuid::new_v4().to_string(),
                    views: vec![ViewState::Search {
                        id: timeline_view_id.clone(),
                        query: "".to_string(),
                        scope: None,
                        order_by: OrderBy::Relevance,
                        view_width_ratio: 1.0,
                        scroll_position: 0,
                    }],
                    focused_view_id: None,
                    pinned_view_ids: pinned_view_ids_search,
                },
            ],
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
pub struct PinnedTabState {
    pub id: String,
    pub views: Vec<ViewState>,
    pub focused_view_id: Option<String>,
    pub pinned_view_ids: HashMap<String, ()>,
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
        view_width_ratio: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
    },
    #[serde(rename_all = "camelCase")]
    Outline {
        id: String,
        outline_id: Option<UUIDv7Base64URL>,
        title: String,
        view_width_ratio: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
    },
    #[serde(rename_all = "camelCase")]
    Document {
        id: String,
        outline_id: Option<UUIDv7Base64URL>,
        title: String,
        view_width_ratio: f64,
        scroll_position: u32,
        focus_position: FocusPosition,
    },
    #[serde(rename_all = "camelCase")]
    Timeline {
        id: String,
        view_width_ratio: f64,
        position: Option<TimelinePosition>,
    },
    #[serde(rename_all = "camelCase")]
    Relation {
        id: String,
        outline_id: UUIDv7Base64URL,
        title: String,
        direction: RelationDirection,
        view_width_ratio: f64,
        scroll_position: u32,
    },
    #[serde(rename_all = "camelCase")]
    Search {
        id: String,
        query: String,
        scope: Option<UUIDv7Base64URL>,
        order_by: OrderBy,
        view_width_ratio: f64,
        scroll_position: u32,
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

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePosition {
    day_start: i64,
    scroll_offset: f64,
}
