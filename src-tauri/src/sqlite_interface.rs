pub mod query;
mod sync;
pub mod table;

use crate::utils::{get_once_lock, set_once_lock};
use anyhow::anyhow;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::fs;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager, Runtime};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");
static POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init_sqlite<R: Runtime>(app_handle: Option<&AppHandle<R>>) -> anyhow::Result<()> {
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
    let _ = set_once_lock(&POOL, pool);

    // if let Some(handle) = app_handle {
    //     sync::start_sync(handle);
    // }

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
    use crate::test::*;
    use crate::{sqlite_interface::table::types::Origin::*, types::NullableBase64String};
    use query::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct Status {
        is_synced: bool,
        is_indexed: bool,
        is_conflicting: bool,
    }

    #[test]
    fn test_init_sqlite() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            println!("test running!");

            let outline = insert_outline(
                app_handle.clone(),
                Some("text"),
                NullableBase64String::none(),
                Local,
            )
            .await
            .unwrap();

            let oplog = select_oplog(outline.id.to_bytes().unwrap()).await.unwrap();
            let blob = oplog.status.as_ref().unwrap();
            let json = serde_sqlite_jsonb::from_slice::<Status>(blob.as_bytes()).unwrap();
            dbg!(&json);

            let outline = insert_outline(
                app_handle.clone(),
                Some("text"),
                NullableBase64String::none(),
                Local,
            )
            .await
            .unwrap();
            let oplog = select_oplog(outline.id.to_bytes().unwrap()).await.unwrap();
            let blob = oplog.status.as_ref().unwrap();
            let json = serde_sqlite_jsonb::from_slice::<Status>(blob.as_bytes()).unwrap();
            dbg!(&json);
            assert!(json.is_conflicting);

            let card = insert_card(
                app_handle.clone(),
                "text",
                Some(outline.id.to_bytes().unwrap()),
                Local,
            )
            .await
            .unwrap();

            let ids: Vec<Vec<u8>> = vec![card.id.to_bytes().unwrap()];
            let results = select_cards(ids).await.unwrap();
            dbg!(results);
        });
    }
}
