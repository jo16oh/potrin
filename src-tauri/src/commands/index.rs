use crate::search_engine;
use search_engine::IndexTarget;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn index(input: Vec<IndexTarget>) -> anyhow::Result<()> {
    search_engine::index(input).await
}
