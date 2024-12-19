use crate::database::query::fetch;
use crate::types::model::Card;
use crate::types::model::Outline;
use crate::types::util::UUIDv7Base64;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_tree<R: Runtime>(
    app_handle: AppHandle<R>,
    id: UUIDv7Base64,
    depth: Option<u32>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let outlines = fetch::outline_trees(pool, &[id], depth).await?;
    let outline_ids = outlines.iter().map(|o| o.id).collect::<Vec<UUIDv7Base64>>();
    let cards = fetch::cards_by_outline_id(pool, &outline_ids).await?;

    Ok((outlines, cards))
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
            let (_, pot) = create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle, pot.id).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>, pot_id: UUIDv7Base64) {
        let outline = create_tree(app_handle, pot_id, None, 2, 0).await;

        let (outlines, cards) = fetch_tree(app_handle.clone(), outline.id, None)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);
    }
}
