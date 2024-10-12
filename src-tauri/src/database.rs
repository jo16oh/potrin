pub mod query;
pub mod table;

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
    use std::sync::RwLock;

    use crate::commands::update_app_state;
    use crate::database::query;
    use crate::state::AppStateValues;
    use crate::types::model::{Card, Outline, Pot, Quote, User};
    use crate::types::state::{AppState, PotState, UserState};
    use crate::types::util::Base64;
    use anyhow::anyhow;
    use chrono::Utc;
    use sqlx::SqlitePool;
    use tauri::Manager;
    use tauri::{test::MockRuntime, AppHandle};

    pub async fn create_tree(
        app_handle: &AppHandle<MockRuntime>,
        parent_id: Option<Base64>,
        limit: u8,
        current: u8,
    ) -> Outline {
        let pool = app_handle
            .try_state::<SqlitePool>()
            .ok_or(anyhow!("failed to get SqlitePool"))
            .unwrap()
            .inner();

        let lock = app_handle
            .try_state::<RwLock<AppState>>()
            .ok_or(anyhow!("failed to get state"))
            .unwrap();

        let pot_id = {
            let app_state = lock.read().map_err(|e| anyhow!(e.to_string())).unwrap();
            let pot = app_state
                .pot
                .as_ref()
                .ok_or(anyhow!("pot state is not set"))
                .unwrap();
            pot.id.clone()
        };

        let outline = Outline::new(parent_id.as_ref());
        query::insert_outline(pool, &outline, &pot_id)
            .await
            .unwrap();

        let card = Card::new(outline.id.clone(), None);
        query::insert_card(pool, &card).await.unwrap();

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
        let pool = app_handle.state::<SqlitePool>().inner();

        let user = User {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            name: "mock_user".to_string(),
        };

        query::insert_user(pool, &user).await.unwrap();

        update_app_state(
            app_handle.clone(),
            AppStateValues::User(Some(UserState {
                id: user.id.clone(),
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

        query::insert_pot(pool, &pot).await.unwrap();

        update_app_state(
            app_handle.clone(),
            AppStateValues::Pot(Some(PotState {
                id: pot.id,
                sync: false,
            })),
        )
        .await
        .unwrap();
    }

    pub async fn insert_quote_without_versioning(
        app_handle: AppHandle<MockRuntime>,
        card_id: &Base64,
        quoted_card_id: &Base64,
    ) -> anyhow::Result<()> {
        let pool = app_handle
            .try_state::<SqlitePool>()
            .ok_or(anyhow!("failed to get SqlitePool"))?
            .inner();
        let lock = app_handle.state::<RwLock<AppState>>().inner();

        let pot_id = {
            let app_state = lock.read().unwrap();
            let pot = app_state.pot.as_ref().unwrap();
            pot.id.clone()
        };

        let now = Utc::now().timestamp_millis();

        let version_id = uuidv7::create_raw().to_vec();

        sqlx::query!(
            r#"
                INSERT INTO versions (id, pot_id, timestamp)
                VALUES (?, ?, ?);
            "#,
            version_id,
            pot_id,
            now
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO quotes (card_id, quoted_card_id, version_id)
                VALUES (?, ?, ?);
            "#,
            card_id,
            quoted_card_id,
            version_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
