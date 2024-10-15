use crate::{
    search_engine::{self, Fields, SearchResult},
    utils::get_rw_state,
};
use tantivy::{query::QueryParser, IndexReader};
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn search<R: Runtime>(
    app_handle: AppHandle<R>,
    query: &str,
    limit: u8,
) -> anyhow::Result<Vec<SearchResult>> {
    let fields_lock = get_rw_state::<R, Fields>(&app_handle)?;
    let reader_lock = get_rw_state::<R, IndexReader>(&app_handle)?;
    let query_parser_lock = get_rw_state::<R, QueryParser>(&app_handle)?;
    let fields = fields_lock.read().await;
    let reader = reader_lock.read().await;
    let query_parser = query_parser_lock.read().await;
    let results = search_engine::search(&fields, &reader, &query_parser, query, limit).await?;

    Ok(results)
}
