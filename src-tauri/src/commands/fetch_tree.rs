use crate::database::query::fetch_cards_by_outline_id;
use crate::database::query::fetch_links;
use crate::database::query::fetch_outline_tree;
use crate::types::model::Card;
use crate::types::model::Link;
use crate::types::model::Outline;
use crate::types::util::Base64;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_tree<R: Runtime>(
    app_handle: AppHandle<R>,
    id: Base64,
    depth: Option<u32>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>, Vec<Link>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let outlines = fetch_outline_tree(&id, depth, pool).await?;
    let outline_ids = outlines.iter().map(|o| &o.id).collect::<Vec<&Base64>>();
    let cards = fetch_cards_by_outline_id(pool, &outline_ids).await?;
    let card_ids = cards.iter().map(|c| &c.id).collect::<Vec<&Base64>>();
    let links = fetch_links(pool, &outline_ids, &card_ids).await?;

    Ok((outlines, cards, links))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::{create_mock_user_and_pot, create_tree};
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_tree() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let outline = create_tree(app_handle, None, 2, 0).await;

        let (outlines, cards, _links) = fetch_tree(app_handle.clone(), outline.id, None)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);
    }
}
