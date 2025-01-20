use crate::{
    database::query::fetch,
    search_engine::{self, OrderBy, SearchIndex, SearchResult},
    types::{
        model::{Outline, Paragraph},
        state::AppState,
        util::UUIDv7Base64URL,
    },
    utils::{get_rw_state, get_state},
};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn search<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    query: &str,
    order_by: OrderBy,
    limit: u8,
) -> eyre::Result<(Vec<Outline>, Vec<Paragraph>, Vec<SearchResult>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let app_state_lock = get_rw_state::<R, AppState>(&app_handle)?;
    let app_state = app_state_lock.read().await;
    let index = get_state::<R, SearchIndex>(&window)?;

    let search_results = search_engine::search(
        index,
        query,
        order_by,
        limit,
        app_state.setting.levenshtein_distance,
    )
    .await?;

    let paragraphs = {
        let paragraph_ids = search_results
            .iter()
            .filter(|r| r.doc_type == "paragraph")
            .map(|r| r.id)
            .collect::<Vec<UUIDv7Base64URL>>();

        fetch::paragraphs_by_id(pool, &paragraph_ids).await?
    };

    let outlines = {
        let outline_ids = [
            search_results
                .iter()
                .filter(|r| r.doc_type == "outline")
                .map(|r| r.id)
                .collect::<Vec<UUIDv7Base64URL>>(),
            paragraphs
                .iter()
                .map(|c| c.id)
                .collect::<Vec<UUIDv7Base64URL>>(),
        ]
        .concat();

        fetch::outlines_by_id(pool, &outline_ids).await?
    };

    eyre::Ok((outlines, paragraphs, search_results))
}
