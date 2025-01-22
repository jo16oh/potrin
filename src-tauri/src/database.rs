pub mod query;

use eyre::eyre;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::any::TypeId;
use std::fs;
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");

pub async fn init<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let pool = if TypeId::of::<R>() == TypeId::of::<MockRuntime>() {
        SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| eyre!(e.to_string()))?
    } else {
        let mut path = app_handle.path().app_data_dir()?;
        path.push("sqlite");

        if !path.exists() {
            fs::create_dir_all(path.as_path())?;
        }

        path.push("data.db");

        let url = path.to_str().ok_or(eyre!("invalid sqlite url"))?;
        Sqlite::create_database(url).await?;

        SqlitePool::connect(url)
            .await
            .map_err(|e| eyre!(e.to_string()))?
    };

    MIGRATOR.run(&pool).await?;
    app_handle.manage::<SqlitePool>(pool);

    Ok(())
}

#[cfg(test)]
pub mod test {
    use crate::commands::upsert_outline::test::upsert_outline;
    use crate::commands::upsert_paragraph::test::upsert_paragraph;
    use crate::types::model::{Outline, Paragraph, Pot};
    use crate::types::util::UUIDv7Base64URL;
    use chrono::Utc;
    use sqlx::SqlitePool;
    use tauri::Manager;
    use tauri::{test::MockRuntime, AppHandle};

    use super::query::insert;

    pub async fn create_tree(
        app_handle: &AppHandle<MockRuntime>,
        pot_id: UUIDv7Base64URL,
        parent_id: Option<UUIDv7Base64URL>,
        limit: u8,
        current: u8,
    ) -> Outline {
        let outline = Outline::new(parent_id);
        upsert_outline(app_handle, pot_id, &outline, vec![])
            .await
            .unwrap();

        let paragraph = Paragraph::new(outline.id, None);
        upsert_paragraph(app_handle, pot_id, &paragraph, vec![])
            .await
            .unwrap();

        if current < limit {
            Box::pin(create_tree(
                app_handle,
                pot_id,
                Some(outline.id),
                limit,
                current + 1,
            ))
            .await;
        }

        outline
    }

    pub async fn create_mock_pot(app_handle: AppHandle<MockRuntime>) -> Pot {
        let pool = app_handle.state::<SqlitePool>().inner();

        let now = Utc::now().timestamp_millis();

        let pot = Pot {
            id: UUIDv7Base64URL::new(),
            name: "mock".to_string(),
            owner: None,
            created_at: Utc::now().timestamp_millis(),
        };
        insert::from_local::pot(pool, &pot, now).await.unwrap();

        pot
    }
}
