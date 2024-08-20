use anyhow::anyhow;
use sqlx::migrate::Migrator;
use sqlx::sqlite::Sqlite;
use sqlx::Pool;
use std::sync::OnceLock;

use crate::utils::set_once_lock;

static MIGRATOR: Migrator = sqlx::migrate!();
static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn init_sqlite() -> anyhow::Result<()> {
    let pool = Pool::<Sqlite>::connect("sqlite::memory:")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    MIGRATOR.run(&pool).await?;
    set_once_lock(&POOL, pool)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test() {
        assert!(init_sqlite().await.is_ok());
    }
}
