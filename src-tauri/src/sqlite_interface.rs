pub mod query;
mod sync;

use crate::utils::{get_once_lock, set_once_lock};
use anyhow::anyhow;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::fs;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");
static POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init_sqlite(app_handle: Option<&AppHandle>) -> anyhow::Result<()> {
    if get_once_lock(&POOL).is_ok() {
        return Ok(());
    }

    let pool = match app_handle {
        Some(handle) => {
            let mut path = handle.path().app_data_dir()?;
            path.extend(["sqlite", "data.db"].iter());

            if !path.parent().unwrap().exists() {
                fs::create_dir_all(path.parent().unwrap())?;
            }

            let url = path.to_str().ok_or(anyhow!("invalid sqlite url"))?;
            Sqlite::create_database(url).await?;

            SqlitePool::connect(url)
                .await
                .map_err(|e| anyhow!(e.to_string()))?
        }
        None => SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| anyhow!(e.to_string()))?,
    };

    MIGRATOR.run(&pool).await?;
    set_once_lock(&POOL, pool)?;

    sync::start_sync(app_handle.unwrap());

    // let client = get_client_info().await?;
    // set_once_lock(&CLIENT_ID, client)?;

    Ok(())
}

// static CLIENT_ID: OnceLock<String> = OnceLock::new();
//
// async fn get_client_info() -> anyhow::Result<String> {
//     struct Result {
//         value: String,
//     }
//
//     let pool = get_once_lock(&POOL)?;
//     let result = sqlx::query_as!(
//         Result,
//         r#"
//             SELECT value FROM store WHERE key = "client_id";
//         "#,
//     )
//     .fetch_one(pool)
//     .await;
//
//     match result {
//         Ok(r) => Ok(r.value),
//         Err(_) => {
//             let id = uuidv7::create();
//             sqlx::query!(
//                 "INSERT INTO store (key, value) VALUES (?, ?);",
//                 "client_id",
//                 id
//             )
//             .execute(pool)
//             .await?;
//
//             Ok(id)
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_init_sqlite() {
        assert!(init_sqlite(None).await.is_ok());
    }
}
