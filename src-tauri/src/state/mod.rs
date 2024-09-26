pub mod types;

use crate::{
    database::types::Base64String,
    utils::{get_once_lock, set_once_lock},
};
use anyhow::anyhow;
use polodb_core::{
    bson::{doc, Bson},
    options::UpdateOptions,
    test_utils::prepare_db,
    Collection, CollectionT, Database,
};
use std::{
    any::TypeId,
    sync::{OnceLock, RwLock},
};
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
        Some(pot) => workspace_state_col.find_one(doc! {"_id": &pot.id})?,
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

    app_handle.manage::<RwLock<AppState>>(RwLock::new(app_state));

    let _ = set_once_lock(&DB, db);
    let _ = set_once_lock(&CLIENT_STATE_COL, client_state_col);
    let _ = set_once_lock(&USER_STATE_COL, user_state_col);
    let _ = set_once_lock(&POT_STATE_COL, pot_state_col);
    let _ = set_once_lock(&WS_STATE_COL, workspace_state_col);
    let _ = set_once_lock(&SETTING_STATE_COL, setting_state_col);

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn set_user_state<R: Runtime>(app_handle: AppHandle<R>, user: UserState) -> anyhow::Result<()> {
    let collection = get_once_lock(&USER_STATE_COL)?;
    collection.update_one_with_options(
        doc! {},
        doc! {
            "$set": &user
        },
        UpdateOptions::builder().upsert(true).build(),
    )?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;
    app_state.user = Some(user);

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn update_user_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: UserStateFields,
) -> anyhow::Result<()> {
    let user_state_col = get_once_lock(&USER_STATE_COL)?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;

    if let Some(ref mut user) = app_state.user {
        user_state_col.update_one(
            doc! {},
            doc! {
                "$set": &value
            },
        )?;

        user.apply(value);

        Ok(())
    } else {
        Err(anyhow!("user state is not set"))
    }
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn set_pot_state<R: Runtime>(
    app_handle: AppHandle<R>,
    pot: PotState,
) -> anyhow::Result<Option<WorkspaceState>> {
    let pot_state_col = get_once_lock(&POT_STATE_COL)?;
    pot_state_col.update_one_with_options(
        doc! {},
        doc! {
            "$set": &pot
        },
        UpdateOptions::builder().upsert(true).build(),
    )?;

    let workspace_state_col = get_once_lock(&WS_STATE_COL)?;
    let workspace = workspace_state_col.find_one(doc! {"_id": &pot.id})?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;
    app_state.pot = Some(pot);
    app_state.workspace = workspace;

    Ok(app_state.workspace.clone())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn update_pot_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: PotStateFields,
) -> anyhow::Result<Option<WorkspaceState>> {
    let pot_state_col = get_once_lock(&POT_STATE_COL)?;
    let workspace_state_col = get_once_lock(&WS_STATE_COL)?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;

    if app_state.pot.is_some() {
        let res = if let PotStateFields::Id(ref id) = value {
            let workspace = workspace_state_col.find_one(doc! {"_id": id})?;
            app_state.workspace = workspace.clone();
            workspace
        } else {
            None
        };

        if let Some(ref mut pot) = app_state.pot {
            pot_state_col.update_one(
                doc! {},
                doc! {
                    "$set": &value
                },
            )?;

            pot.apply(value);
        }

        Ok(res)
    } else {
        Err(anyhow!("pot state is not set"))
    }
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn set_workspace_state<R: Runtime>(
    app_handle: AppHandle<R>,
    workspace: WorkspaceState,
) -> anyhow::Result<()> {
    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;

    let pot_id = match &app_state.pot {
        Some(pot) => Ok(&pot.id),
        None => Err(anyhow!("pot state is not set")),
    }?;

    let workspace_state_col = get_once_lock(&WS_STATE_COL)?;
    workspace_state_col.update_one_with_options(
        doc! {"_id": pot_id},
        doc! {
            "$set": &workspace
        },
        UpdateOptions::builder().upsert(true).build(),
    )?;

    app_state.workspace = Some(workspace);

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn update_workspace_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: WorkspaceStateFields,
) -> anyhow::Result<()> {
    let workspace_state_col = get_once_lock(&WS_STATE_COL)?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;

    let pot_id = match &app_state.pot {
        Some(pot) => Ok(&pot.id),
        None => Err(anyhow!("pot state is not set")),
    }?;

    if app_state.workspace.is_some() {
        workspace_state_col.update_one(
            doc! {"_id": pot_id},
            doc! {
                "$set": &value
            },
        )?;

        if let Some(ref mut workspace) = app_state.workspace {
            workspace.apply(value);
        };

        Ok(())
    } else {
        Err(anyhow!("workspace state is not set"))
    }
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub fn set_setting_state<R: Runtime>(
    app_handle: AppHandle<R>,
    setting: SettingState,
) -> anyhow::Result<()> {
    let setting_state_col = get_once_lock(&SETTING_STATE_COL)?;
    setting_state_col.update_one_with_options(
        doc! {},
        doc! {
            "$set": &setting
        },
        UpdateOptions::builder().upsert(true).build(),
    )?;

    let lock = app_handle
        .try_state::<RwLock<AppState>>()
        .ok_or(anyhow!("failed to get state"))?;
    let mut app_state = lock.write().map_err(|e| anyhow!(e.to_string()))?;
    app_state.setting = setting;

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
            let app_state = app_handle.try_state::<RwLock<AppState>>();
            assert!(app_state.is_some());
        });
    }

    #[test]
    fn test_user_state() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let collection = get_once_lock(&USER_STATE_COL).unwrap();

            let user = UserState {
                id: Base64String::from_bytes(uuidv7::create_raw().to_vec()),
                name: "name".to_string(),
            };

            set_user_state(app_handle.clone(), user.clone()).unwrap();

            assert_eq!(
                collection.find_one(doc! {}).unwrap().unwrap().name,
                user.name
            );

            let value = UserStateFields::Name("updated".to_string());
            update_user_state(app_handle.clone(), value).unwrap();

            let result = collection.find_one(doc! {}).unwrap().unwrap();

            assert_eq!(result.name, "updated".to_string());
            assert_eq!(result.id.to_string(), user.id.to_string());
        });
    }

    #[test]
    fn test_pot_state() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let pot = PotState {
                id: Base64String::from_bytes(uuidv7::create_raw().to_vec()),
                sync: true,
            };
            set_pot_state(app_handle.clone(), pot.clone()).unwrap();
            let collection = get_once_lock(&POT_STATE_COL).unwrap();

            assert_eq!(
                collection.find_one(doc! {}).unwrap().unwrap().sync,
                pot.sync
            );
        });
    }
}
