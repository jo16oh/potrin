use crate::database::query::fetch;
use crate::types::model::Outline;
use crate::types::model::Paragraph;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_tree<R: Runtime>(
    app_handle: AppHandle<R>,
    id: UUIDv7Base64URL,
    depth: Option<u32>,
) -> eyre::Result<(Vec<Outline>, Vec<Paragraph>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let outlines = fetch::outline_trees(pool, &[id], depth).await?;
    let outline_ids = outlines
        .iter()
        .map(|o| o.id)
        .collect::<Vec<UUIDv7Base64URL>>();
    let paragraphs = fetch::paragraphs_by_outline_id(pool, &outline_ids).await?;

    eyre::Ok((outlines, paragraphs))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::{create_mock_pot, create_tree};
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_tree() {
        run_in_mock_app!(test)
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot = create_mock_pot(app_handle).await;
        let outline = create_tree(app_handle, pot.id, None, 2, 0).await;

        let (outlines, paragraphs) = fetch_tree(app_handle.clone(), outline.id, None)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(paragraphs.len(), 3);

        Ok(())
    }
}
