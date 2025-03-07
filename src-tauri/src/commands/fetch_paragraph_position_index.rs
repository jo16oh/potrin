use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use crate::{database::query::fetch, types::model::ParagraphPositionIndex};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_paragraph_position_index<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<UUIDv7Base64URL>,
    paragraph_ids: Vec<UUIDv7Base64URL>,
) -> eyre::Result<ParagraphPositionIndex> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::paragraph_position_index(pool, &outline_ids, &paragraph_ids).await
}
