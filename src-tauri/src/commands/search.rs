use crate::{
    database::query::fetch,
    search_engine::{self, OrderBy, SearchIndex},
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
    scope: Option<Vec<UUIDv7Base64URL>>,
    order_by: OrderBy,
    offset: u32,
    limit: u32,
) -> eyre::Result<(Vec<Outline>, Vec<Paragraph>, Vec<UUIDv7Base64URL>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let pot_id = window.label().try_into()?;
    let app_state_lock = get_rw_state::<R, AppState>(&app_handle)?;
    let app_state = app_state_lock.read().await;
    let index = get_state::<R, SearchIndex>(&window)?;

    let search_results = search_engine::search(
        index,
        query,
        scope,
        order_by,
        offset,
        limit,
        app_state.setting.search.fuzziness,
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
                .map(|c| c.outline_id)
                .collect::<Vec<UUIDv7Base64URL>>(),
        ]
        .concat();

        fetch::outlines_with_path_by_id(pool, pot_id, &outline_ids).await?
    };

    eyre::Ok((
        outlines,
        paragraphs,
        search_results.iter().map(|r| r.id).collect(),
    ))
}
