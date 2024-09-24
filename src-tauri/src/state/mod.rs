mod types;

use crate::{database::types::Base64String, utils::set_once_lock};
use polodb_core::{
    bson::{bson, doc, Bson},
    test_utils::prepare_db,
    Collection, CollectionT, Database,
};
use std::{any::TypeId, sync::OnceLock};
use tauri::{test::MockRuntime, AppHandle, Manager, Runtime};
use types::*;

static DB: OnceLock<Database> = OnceLock::new();
static CLIENT_STATE_COL: OnceLock<Collection<ClientState>> = OnceLock::new();
static USER_STATE_COL: OnceLock<Collection<UserState>> = OnceLock::new();
static POT_STATE_COL: OnceLock<Collection<PotState>> = OnceLock::new();
static WS_STATE_COL: OnceLock<Collection<WorkspaceState>> = OnceLock::new();
static SETTING_STATE_COL: OnceLock<Collection<SettingState>> = OnceLock::new();

impl From<Base64String> for Bson {
    fn from(value: Base64String) -> Self {
        Bson::from(value.to_string())
    }
}

pub fn init<R: Runtime>(app_handle: AppHandle<R>) -> anyhow::Result<()> {
    let db = if TypeId::of::<R>() == TypeId::of::<MockRuntime>() {
        prepare_db(uuidv7::create().as_str())?
    } else {
        let mut path = app_handle.path().app_data_dir()?;
        path.push("state");
        Database::open_path(path)?
    };

    let client_state_col = db.collection::<ClientState>("ClientState");
    let user_state_col = db.collection::<UserState>("UserState");
    let pot_state_col = db.collection::<PotState>("PotState");
    let workspace_state_col = db.collection::<WorkspaceState>("WorkspaceState");
    let setting_state_col = db.collection::<SettingState>("SettingState");

    let client = match client_state_col.find_one(doc! {})? {
        Some(client) => client,
        None => {
            let client = ClientState {
                id: Base64String::from_bytes(uuidv7::create_raw().to_vec()),
            };

            client_state_col.insert_one(&client)?;
            client
        }
    };

    let user = user_state_col.find_one(doc! {})?;
    let pot = pot_state_col.find_one(doc! {})?;

    let workspace = match &pot {
        Some(pot) => workspace_state_col.find_one(doc! {"_id": bson!(&pot.id)})?,
        None => None,
    };

    let setting = match setting_state_col.find_one(doc! {})? {
        Some(setting) => setting,
        None => SettingState {},
    };

    let app_state = AppState {
        client,
        user,
        pot,
        workspace,
        setting,
    };

    app_handle.manage::<AppState>(app_state);

    let _ = set_once_lock(&DB, db);
    let _ = set_once_lock(&CLIENT_STATE_COL, client_state_col);
    let _ = set_once_lock(&USER_STATE_COL, user_state_col);
    let _ = set_once_lock(&POT_STATE_COL, pot_state_col);
    let _ = set_once_lock(&WS_STATE_COL, workspace_state_col);
    let _ = set_once_lock(&SETTING_STATE_COL, setting_state_col);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use tauri::AppHandle;

    #[test]
    fn test_init() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let app_state = app_handle.try_state::<AppState>();
            assert!(app_state.is_some());
        });
    }
}
