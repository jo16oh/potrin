pub mod query;
mod sync;
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

    // if let Some(handle) = app_handle {
    //     sync::start_sync(handle);
    // }

    // let client = get_client_info().await?;
    // set_once_lock(&CLIENT_ID, client)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::database::query::{insert_card, insert_outline};
    use crate::database::types::Base64String;
    use tauri::{test::MockRuntime, AppHandle};

    use super::table::{Card, CardYUpdate, Outline, OutlineYUpdate};

    pub async fn create_tree(
        app_handle: &AppHandle<MockRuntime>,
        parent_id: Option<Base64String>,
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
}
