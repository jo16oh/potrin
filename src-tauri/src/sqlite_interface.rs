pub mod query;
mod sync;
pub mod table;

use anyhow::anyhow;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::any::TypeId;
use std::fs;
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");

pub async fn init_sqlite<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<()> {
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
    use crate::types::Base64String;
    use crate::{sqlite_interface::table::types::Origin::*, types::NullableBase64String};
    use crate::{test::*, OutlinesTableChangeEvent};
    use query::*;
    use serde::{Deserialize, Serialize};
    use tauri::Listener;
    use tauri_specta::Event;

    #[derive(Serialize, Deserialize, Debug)]
    struct Status {
        is_synced: bool,
        is_indexed: bool,
        is_conflicting: bool,
    }

    #[test]
    fn test_init_sqlite() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let handle_clone = app_handle.clone();
            OutlinesTableChangeEvent::listen(app_handle, move |event| {
                println!("unlisten");
                handle_clone.unlisten(event.id);
            });

            OutlinesTableChangeEvent::listen(app_handle, move |event| {
                println!("outline changed!");
                println!("{:?}", event.payload.rows_changed);
            });

            let outline = insert_outline(
                app_handle.clone(),
                Some("text"),
                NullableBase64String::none(),
                Local,
            )
            .await
            .unwrap();

            let oplog = select_oplog(app_handle.clone(), outline.id.to_bytes().unwrap())
                .await
                .unwrap();
            let blob = oplog.status.as_ref().unwrap();
            let json =
                serde_sqlite_jsonb::from_slice::<Status>(blob.to_bytes().unwrap().as_slice());
            assert!(json.is_ok());

            let outline = insert_outline(
                app_handle.clone(),
                Some("text"),
                NullableBase64String::none(),
                Local,
            )
            .await
            .unwrap();

            let oplog = select_oplog(app_handle.clone(), outline.id.to_bytes().unwrap())
                .await
                .unwrap();
            let blob = oplog.status.as_ref().unwrap();
            let base64 = Base64String::from(blob.to_bytes().unwrap());
            let blob = base64.to_bytes().unwrap();
            let json = serde_sqlite_jsonb::from_slice::<Status>(&blob).unwrap();
            assert!(json.is_conflicting);

            let card = insert_card(
                app_handle.clone(),
                "text",
                NullableBase64String::from(outline.id),
                Local,
            )
            .await
            .unwrap();

            let ids: Vec<Base64String> = vec![card.id];
            let results = select_cards(app_handle.clone(), ids).await.unwrap();
            assert!(!results.is_empty());
        });
    }
}
