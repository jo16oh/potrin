use crate::database::table::CardsTable;
use crate::database::table::OutlinesTable;
use crate::database::types::Base64String;
use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde::Serialize;
use sqlx::query_as;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

use super::fetch_breadcrumbs;
use super::fetch_breadcrumbs::Breadcrumb;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub enum TlOption {
    CreatedAt,
    UpdatedAt,
    Both,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    from: DateTime<Utc>,
    option: TlOption,
) -> anyhow::Result<(Vec<OutlinesTable>, Vec<CardsTable>, Vec<Breadcrumb>)> {
    let to = (from + Duration::days(1)).timestamp_millis();
    let from = from.timestamp_millis();

    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let cards = match option {
        TlOption::CreatedAt => {
            sqlx::query_as!(
                CardsTable,
                r#"
                    SELECT *
                    FROM cards
                    WHERE ? <= created_at AND created_at < ? AND is_deleted = false;
                "#,
                from,
                to,
            )
            .fetch_all(pool)
            .await
        }
        TlOption::UpdatedAt => {
            sqlx::query_as!(
                CardsTable,
                r#"
                    SELECT *
                    FROM cards
                    WHERE ? <= updated_at AND updated_at < ? AND is_deleted = false;
                "#,
                from,
                to,
            )
            .fetch_all(pool)
            .await
        }
        TlOption::Both => {
            sqlx::query_as!(
                CardsTable,
                r#"
                    SELECT *
                    FROM cards
                    WHERE
                        ((? <= updated_at AND updated_at < ?) OR (? <= created_at AND created_at < ?))
                        AND is_deleted = false;
                "#,
                from,
                to,
                from,
                to
            )
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| anyhow!(e.to_string()))?;

    let ids = cards
        .iter()
        .map(|c| c.id.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let outlines = query_as!(
        OutlinesTable,
        r#"
            SELECT *
            FROM outlines
            WHERE id IN (?) AND is_deleted = false;
        "#,
        ids
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let parent_ids = outlines
        .iter()
        .filter_map(|o| o.parent_id.as_ref())
        .collect::<Vec<&Base64String>>();

    let breadcrumbs = fetch_breadcrumbs(parent_ids, pool).await?;
    Ok((outlines, cards, breadcrumbs))
}