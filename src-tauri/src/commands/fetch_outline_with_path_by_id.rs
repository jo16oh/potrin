use std::collections::VecDeque;

use crate::database::query::fetch;
use crate::types::model::Outline;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_outline_with_path_by_id<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    id: UUIDv7Base64URL,
) -> eyre::Result<Option<Outline>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let pot_id = window.label().try_into()?;

    let mut outlines: VecDeque<Outline> = fetch::outlines_with_path_by_id(pool, pot_id, &[id])
        .await?
        .into();

    eyre::Ok(outlines.pop_front())
}
