use crate::types::util::Base64;
use crate::utils::get_state;
use crate::{types::state::*, utils::get_rw_state};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{async_runtime::RwLock, AppHandle, Manager, Runtime};

struct QueryResult {
    value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum AppStateValues {
    User(Option<UserState>),
    Pot(Option<PotState>),
    Workspace(Option<WorkspaceState>),
    Tabs(Vec<TabState>),
    Setting(SettingState),
}

pub async fn init<R: Runtime>(app_handle: AppHandle<R>) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let initial = sqlx::query_as!(
        QueryResult,
        r#"
            SELECT value
            FROM kv
            WHERE key = ?;
        "#,
        "app_state"
    )
    .fetch_optional(pool)
    .await?;

    let app_state = match initial {
        Some(result) => serde_sqlite_jsonb::from_slice::<AppState>(&result.value)?,
        None => {
            let app_state = AppState {
                client: ClientState {
                    id: Base64::from(uuidv7::create_raw().to_vec()),
                },
                user: None,
                pot: None,
                workspace: None,
                setting: SettingState {},
            };

            let jsonb = serde_sqlite_jsonb::to_vec(&app_state)?;

            sqlx::query!(
                r#"
                    INSERT INTO kv (key, value)
                    VALUES (?, ?);
                "#,
                "app_state",
                jsonb
            )
            .execute(pool)
            .await?;

            app_state
        }
    };

    app_handle.manage::<RwLock<AppState>>(RwLock::new(app_state));

    Ok(())
}

pub async fn update_app_state<R: Runtime>(
    app_handle: AppHandle<R>,
    value: AppStateValues,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    match value {
        AppStateValues::User(user_state) => {
            let jsonb = serde_sqlite_jsonb::to_vec(&user_state)?;
            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.user',
                        ?
                    )
                    WHERE key = "app_state";
                "#,
                jsonb
            )
            .execute(pool)
            .await?;

            let lock = get_rw_state::<R, AppState>(&app_handle)?;
            let mut app_state = lock.write().await;
            app_state.user = user_state;
        }
        AppStateValues::Pot(pot_state) => {
            let workspace_state = match pot_state {
                Some(ref pot) => {
                    let id = pot.id.clone();
                    sqlx::query_as!(
                        QueryResult,
                        r#"
                            SELECT value
                            FROM workspaces
                            WHERE pot_id = ?;
                        "#,
                        id
                    )
                    .fetch_optional(pool)
                    .await?
                    .map(|r| serde_sqlite_jsonb::from_slice(&r.value))
                    .transpose()?
                }
                None => None,
            };

            let value = serde_sqlite_jsonb::to_vec(&pot_state)?;
            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.pot',
                        ?
                    )
                    WHERE key = "app_state";
                "#,
                value
            )
            .execute(pool)
            .await?;

            let value = serde_sqlite_jsonb::to_vec(&workspace_state)?;
            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.workspace',
                        ?
                    )
                    WHERE key = "app_state";
                "#,
                value
            )
            .execute(pool)
            .await?;

            let lock = get_rw_state::<R, AppState>(&app_handle)?;
            let mut app_state = lock.write().await;
            app_state.pot = pot_state;
            app_state.workspace = workspace_state;
        }
        AppStateValues::Workspace(workspace_state) => {
            let lock = get_rw_state::<R, AppState>(&app_handle)?;

            let value = serde_sqlite_jsonb::to_vec(&workspace_state)?;
            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.workspace',
                        ?
                    )
                    WHERE key = "app_state";
                "#,
                value
            )
            .execute(pool)
            .await?;

            if let Some(ref workspace) = workspace_state {
                let app_state = lock.read().await;
                let pot_id = &app_state
                    .pot
                    .as_ref()
                    .ok_or(anyhow!("pot state is not set"))?
                    .id;

                let value = serde_sqlite_jsonb::to_vec(&workspace)?;

                sqlx::query!(
                    r#"
                        INSERT INTO workspaces (pot_id, value)
                        VALUES (?, ?)
                        ON CONFLICT (pot_id)
                        DO UPDATE
                        SET value = excluded.value;
                    "#,
                    pot_id,
                    value
                )
                .execute(pool)
                .await?;
            }

            let mut app_state = lock.write().await;
            app_state.workspace = workspace_state;
        }
        AppStateValues::Tabs(tabs) => {
            let value = serde_sqlite_jsonb::to_vec(&tabs)?;

            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.workspace',
                        jsonb_set(
                            json_extract(
                                (
                                    SELECT value
                                    FROM kv
                                    WHERE key = "app_state"
                                ),
                                '$.workspace'
                            ),
                            "$.tabs",
                            ?
                        )
                    )
                    WHERE key = "app_state";
                "#,
                value
            )
            .execute(pool)
            .await?;

            let lock = get_rw_state::<R, AppState>(&app_handle)?;

            {
                let app_state = lock.read().await;
                let pot_id = &app_state
                    .pot
                    .as_ref()
                    .ok_or(anyhow!("pot state is not set"))?
                    .id;

                sqlx::query!(
                    r#"
                        INSERT INTO workspaces (pot_id, value)
                        VALUES (?, ?)
                        ON CONFLICT (pot_id)
                        DO UPDATE
                        SET value = excluded.value;
                    "#,
                    pot_id,
                    value
                )
                .execute(pool)
                .await?;
            }

            let mut app_state = lock.write().await;
            app_state
                .workspace
                .as_mut()
                .ok_or(anyhow!("workspace state is not set"))?
                .tabs = tabs;
        }
        AppStateValues::Setting(setting) => {
            let jsonb = serde_sqlite_jsonb::to_vec(&setting)?;
            sqlx::query!(
                r#"
                    UPDATE kv
                    SET value = jsonb_set(
                        (
                            SELECT value
                            FROM kv
                            WHERE key = "app_state"
                        ),
                        '$.setting',
                        ?
                    )
                    WHERE key = "app_state";
                "#,
                jsonb
            )
            .execute(pool)
            .await?;

            let lock = get_rw_state::<R, AppState>(&app_handle)?;
            let mut app_state = lock.write().await;
            app_state.setting = setting;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_mock_user_and_pot;
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;
    use tauri::AppHandle;

    #[test]
    fn test_init() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let app_state = get_rw_state::<MockRuntime, AppState>(app_handle);
            assert!(app_state.is_ok());
        });
    }

    #[test]
    fn test_user_state() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let pool = app_handle.state::<SqlitePool>().inner();

            let user = UserState {
                id: Base64::from(uuidv7::create_raw().to_vec()),
                name: "updated".to_string(),
            };

            update_app_state(app_handle.clone(), AppStateValues::User(Some(user)))
                .await
                .unwrap();

            let result = sqlx::query_as!(
                QueryResult,
                r#"
                    SELECT value
                    FROM kv
                    WHERE key = ?;
                "#,
                "app_state"
            )
            .fetch_one(pool)
            .await
            .unwrap();

            let app_state: AppState = serde_sqlite_jsonb::from_slice(&result.value).unwrap();
            assert_eq!(app_state.user.unwrap().name, "updated");
        });
    }

    #[test]
    fn test_pot_and_workspace_state() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let pool = app_handle.state::<SqlitePool>().inner();

            create_mock_user_and_pot(app_handle.clone()).await;

            // update pot state
            let pot = PotState {
                id: Base64::from(uuidv7::create_raw().to_vec()),
                sync: true,
            };

            update_app_state(app_handle.clone(), AppStateValues::Pot(Some(pot.clone())))
                .await
                .unwrap();

            let app_state = get_app_state(pool).await;

            assert_eq!(app_state.pot.unwrap().id, pot.id);
            assert!(app_state.workspace.is_none());

            // update workspace state
            let workspace = WorkspaceState {
                tabs: vec![TabState {
                    id: Base64::from(uuidv7::create_raw().to_vec()),
                    view: "view".to_string(),
                    scroll_pos: 32,
                }],
                focused_tab_idx: Some(1),
            };

            update_app_state(
                app_handle.clone(),
                AppStateValues::Workspace(Some(workspace)),
            )
            .await
            .unwrap();

            let app_state = get_app_state(pool).await;

            assert!(app_state.workspace.is_some());

            // is workspace updated after pot state changed?
            let pot2 = PotState {
                id: Base64::from(uuidv7::create_raw().to_vec()),
                sync: true,
            };

            update_app_state(app_handle.clone(), AppStateValues::Pot(Some(pot2.clone())))
                .await
                .unwrap();

            let app_state = get_app_state(pool).await;
            assert_eq!(app_state.pot.unwrap().id, pot2.id);
            assert!(app_state.workspace.is_none());

            update_app_state(app_handle.clone(), AppStateValues::Pot(Some(pot.clone())))
                .await
                .unwrap();

            let app_state = get_app_state(pool).await;
            assert_eq!(app_state.pot.unwrap().id, pot.id);
            assert!(app_state.workspace.is_some());
        });
    }

    async fn get_app_state(pool: &SqlitePool) -> AppState {
        sqlx::query_as!(
            QueryResult,
            r#"
                SELECT value
                FROM kv
                WHERE key = ?;
            "#,
            "app_state"
        )
        .fetch_one(pool)
        .await
        .map(|r| serde_sqlite_jsonb::from_slice(&r.value).unwrap())
        .unwrap()
    }
}
