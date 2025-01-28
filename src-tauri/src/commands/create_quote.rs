use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime, Window};

use crate::{
    database::query::fetch,
    types::{model::Quote, util::UUIDv7Base64URL},
    utils::get_state,
};

use super::create_version::create_version_impl;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn create_quote<R: Runtime>(
    app_handle: AppHandle<R>,
    window: Window<R>,
    paragraph_id: UUIDv7Base64URL,
) -> eyre::Result<Quote> {
    let version_id = UUIDv7Base64URL::new();
    let pot_id: UUIDv7Base64URL = window.label().try_into()?;

    create_quote_impl(&app_handle, pot_id, paragraph_id, version_id).await
}

async fn create_quote_impl<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    paragraph_id: UUIDv7Base64URL,
    version_id: UUIDv7Base64URL,
) -> eyre::Result<Quote> {
    let pool = get_state::<_, SqlitePool>(app_handle)?;

    create_version_impl(app_handle, pot_id, version_id).await?;
    let (doc, path) = fetch::paragraphs_doc_and_path_by_id(pool, paragraph_id).await?;

    eyre::Ok(Quote {
        id: paragraph_id,
        latest_doc: Some(doc.clone()),
        doc,
        version_id,
        path,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        commands::{
            upsert_outline::test::upsert_outline, upsert_paragraph::test::upsert_paragraph,
        },
        database::test::create_mock_pot,
        run_in_mock_app,
        types::model::{Outline, Paragraph},
    };
    use std::{thread::sleep, time::Duration};
    use tauri::{test::MockRuntime, AppHandle};

    #[test]
    fn test_create_quote() {
        run_in_mock_app!(test_impl)
    }

    async fn test_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot = create_mock_pot(app_handle.clone()).await;

        let o = Outline::new(None);
        let p = Paragraph::new(o.id, None);

        upsert_outline(app_handle, pot.id, &o, vec![]).await?;
        upsert_paragraph(app_handle, pot.id, &p, vec![]).await?;

        // wait until the path is constructed
        sleep(Duration::from_millis(100));

        let version_id = UUIDv7Base64URL::new();

        let quote = create_quote_impl(app_handle, pot.id, p.id, version_id).await?;

        assert_eq!(quote.id, p.id);
        assert_eq!(quote.version_id, version_id);

        Ok(())
    }
}
