use crate::{types::Base64String, CardsTable};
use anyhow::anyhow;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn select_cards<R: Runtime>(
    app_handle: AppHandle<R>,
    ids: Vec<Base64String>,
) -> anyhow::Result<Vec<CardsTable>> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT * FROM cards WHERE id IN (");

    let mut separated = query_builder.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    // dbg!(query_builder.sql());

    query_builder
        .build_query_as::<CardsTable>()
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!(e.to_string()))
}
