use crate::database::types::Base64String;
use macros::Bson;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    pub client: ClientState,
    pub user: Option<UserState>,
    pub pot: Option<PotState>,
    pub workspace: Option<WorkspaceState>,
    pub setting: SettingState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Bson)]
#[macros::keys]
pub struct ClientState {
    pub id: Base64String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Bson)]
#[macros::keys]
pub struct UserState {
    pub id: Base64String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Bson)]
#[macros::keys]
pub struct PotState {
    pub id: Base64String,
    pub sync: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkspaceState {
    pub tabs: Vec<TabState>,
    pub focused_tab_idx: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Bson)]
#[macros::keys]
pub struct TabState {
    pub id: Base64String,
    pub view: String,
    pub scroll_pos: i64,
}

#[derive(Serialize, Deserialize, Clone, Bson)]
pub struct SettingState {}
