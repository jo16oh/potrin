use crate::{
    database::query::fetch,
    search_engine::{self, OrderBy},
    state::SearchIndices,
    types::{
        model::{Outline, Paragraph, ParagraphPositionIndex},
        state::AppState,
        util::UUIDv7Base64URL,
    },
    utils::{get_rw_state, get_state},
};
use eyre::OptionExt;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

type SearchResult = (
    Vec<Outline>,
    Vec<Paragraph>,
    Vec<UUIDv7Base64URL>,
    ParagraphPositionIndex,
);

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
) -> eyre::Result<SearchResult> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;
    let pot_id = window.label().try_into()?;
    let app_state_lock = get_rw_state::<R, AppState>(&app_handle)?;
    let app_state = app_state_lock.read().await;
    let search_indices_lock = get_rw_state::<R, SearchIndices>(&window)?;
    let search_indices = search_indices_lock.read().await;
    let search_index = search_indices
        .get(&pot_id)
        .ok_or_eyre("search indices is not set")?;

    let search_results = search_engine::search(
        search_index,
        query,
        scope,
        order_by,
        offset,
        limit,
        app_state.setting.search.fuzziness,
    )
    .await?;

    let paragraph_ids = search_results
        .iter()
        .filter(|r| r.doc_type == "paragraph")
        .map(|r| r.id)
        .collect::<Vec<UUIDv7Base64URL>>();

    let paragraphs = fetch::paragraphs_by_id(pool, &paragraph_ids).await?;

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

    let outlines = fetch::outlines_with_path_by_id(pool, pot_id, &outline_ids).await?;

    let paragraph_position_index =
        fetch::paragraph_position_index(pool, &outline_ids, &paragraph_ids).await?;

    eyre::Ok((
        outlines,
        paragraphs,
        search_results.iter().map(|r| r.id).collect(),
        paragraph_position_index,
    ))
}
