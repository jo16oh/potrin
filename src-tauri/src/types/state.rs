use serde::{Deserialize, Serialize};

use super::util::Base64;

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct AppState {
    pub client: ClientState,
    pub user: Option<UserState>,
    pub pot: Option<PotState>,
    pub workspace: Option<WorkspaceState>,
    pub setting: SettingState,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct ClientState {
    pub id: Base64,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct UserState {
    pub id: Base64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct PotState {
    pub id: Base64,
    pub sync: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct WorkspaceState {
    pub tabs: Vec<TabState>,
    pub focused_tab_idx: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct TabState {
    pub id: Base64,
    pub view: String,
    pub scroll_pos: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct SettingState {}
