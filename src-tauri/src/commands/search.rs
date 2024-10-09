use crate::search_engine;
use search_engine::SearchResult;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn search(query: &str, pot_id: &str, limit: u8) -> anyhow::Result<Vec<SearchResult>> {
    search_engine::search(query, pot_id, limit).await
}
