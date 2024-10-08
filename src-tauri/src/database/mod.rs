pub mod query;
pub mod table;
pub mod types;

use anyhow::anyhow;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::any::TypeId;
use std::fs;
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");

pub async fn init<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<()> {
    let pool = if TypeId::of::<R>() == TypeId::of::<MockRuntime>() {
        SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| anyhow!(e.to_string()))?
    } else {
        let mut path = app_handle.path().app_data_dir()?;
        path.extend(["sqlite", "data.db"].iter());

        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }

        let url = path.to_str().ok_or(anyhow!("invalid sqlite url"))?;
        Sqlite::create_database(url).await?;

        SqlitePool::connect(url)
            .await
            .map_err(|e| anyhow!(e.to_string()))?
    };

    MIGRATOR.run(&pool).await?;
    app_handle.manage::<SqlitePool>(pool);

    Ok(())
}

#[cfg(test)]
pub mod test {
    use crate::database::query::{insert_card, insert_outline};
    use crate::database::types::Base64;
    use crate::state::types::{PotState, UserState};
    use crate::state::update_app_state;
    use tauri::{test::MockRuntime, AppHandle};
    use super::query::{insert_pot, insert_user};
    use super::table::{Card, CardYUpdate, Outline, OutlineYUpdate, Pot, User};

    pub async fn create_tree(
        app_handle: &AppHandle<MockRuntime>,
        parent_id: Option<Base64>,
        limit: u8,
        current: u8,
    ) -> Outline {
        let outline = insert_outline(
            app_handle.clone(),
            Outline::new(parent_id.as_ref()),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        insert_card(
            app_handle.clone(),
            Card::new(outline.id.clone()),
            vec![CardYUpdate::new()],
        )
        .await
        .unwrap();

        if current < limit {
            Box::pin(create_tree(
                app_handle,
                Some(outline.id.clone()),
                limit,
                current + 1,
            ))
            .await;
        }

        outline
    }

    pub async fn create_mock_user_and_pot(app_handle: AppHandle<MockRuntime>) {
        let user = User {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            name: "mock_user".to_string(),
        };

        insert_user(app_handle.clone(), user.clone()).await.unwrap();

        update_app_state(
            app_handle.clone(),
            crate::state::AppStateValues::User(Some(UserState {
                id: user.id.clone().to_string(),
                name: user.name.clone(),
            })),
        )
        .await
        .unwrap();

        let pot = Pot {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            name: "mock".to_string(),
            owner: user.id.clone(),
        };

        insert_pot(app_handle.clone(), pot.clone()).await.unwrap();

        update_app_state(
            app_handle.clone(),
            crate::state::AppStateValues::Pot(Some(PotState {
                id: pot.id.to_string(),
                sync: false,
            })),
        )
        .await
        .unwrap();
    }
}
