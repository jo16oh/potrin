use serde::{Deserialize, Serialize};

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
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct UserState {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct PotState {
    pub id: String,
    pub sync: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct WorkspaceState {
    pub tabs: Vec<TabState>,
    pub focused_tab_idx: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct TabState {
    pub id: String,
    pub view: String,
    pub scroll_pos: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct SettingState {}
