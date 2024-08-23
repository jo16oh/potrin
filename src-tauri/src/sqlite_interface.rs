use anyhow::anyhow;
use chrono::Utc;
use serde::Serialize;
use specta::Type;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{Sqlite, SqlitePool};
use std::fs;
use std::{path::PathBuf, sync::OnceLock};

use crate::utils::{get_once_lock, set_once_lock};

static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");
static POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init_sqlite(data_dir_path: Option<&PathBuf>) -> anyhow::Result<()> {
    if get_once_lock(&POOL).is_ok() {
        return Ok(());
    }

    let pool = match data_dir_path {
        Some(p) => {
            let mut path = p.to_owned();
            path.extend(["sqlite", "data.db"].iter());

            if !path.parent().ok_or(anyhow!("unwrap failed"))?.exists() {
                fs::create_dir_all(&path)?;
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

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn insert(text: &str) -> anyhow::Result<String> {
    let pool = get_once_lock(&POOL)?;
    let id = uuidv7::create().to_string();
    let now = Utc::now().timestamp_millis();
    sqlx::query!(
        "INSERT INTO outlines (id, parent, text, created_at, updated_at) VALUES (?, ?, ?, ?, ?);",
        id,
        Option::<String>::None,
        text,
        now,
        now
    )
    .execute(pool)
    .await?;

    Ok(id)
}

#[derive(Serialize, Type)]
pub struct RawOutline {
    id: String,
    parent: Option<String>,
    text: String,
    created_at: i64,
    updated_at: i64,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select(id: &str) -> anyhow::Result<RawOutline> {
    let pool = get_once_lock(&POOL)?;
    sqlx::query_as!(
        RawOutline,
        r#"SELECT id, parent, text, created_at, updated_at FROM outlines WHERE id = ?;"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_all() -> anyhow::Result<Vec<RawOutline>> {
    let pool = get_once_lock(&POOL)?;
    sqlx::query_as!(
        RawOutline,
        r#"SELECT id, parent, text, created_at, updated_at FROM outlines;"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_sqlite() {
        assert!(init_sqlite(None).await.is_ok());

        let id = insert("lorem ipsum").await.unwrap();
        let result = select(&id).await;
        assert!(result.is_ok());
    }
}
