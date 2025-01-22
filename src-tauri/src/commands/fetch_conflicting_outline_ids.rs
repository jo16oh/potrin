use crate::database::query::fetch;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_conflicting_outline_ids<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_id: UUIDv7Base64URL,
    parent_id: Option<UUIDv7Base64URL>,
    text: &str,
) -> eyre::Result<Vec<(UUIDv7Base64URL, String)>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    fetch::conflicting_outline_ids(pool, outline_id, parent_id, text).await
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        commands::upsert_outline::test::upsert_outline,
        database::test::create_mock_pot,
        run_in_mock_app,
        types::{
            model::{Links, Outline},
            util::UUIDv7Base64URL,
        },
    };
    use tauri::{test::MockRuntime, AppHandle};

    #[test]
    fn test() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let pot = create_mock_pot(app_handle.clone()).await;

            let outline = |text: &str| -> Outline {
                let now = chrono::Utc::now().timestamp_millis();
                Outline {
                    id: UUIDv7Base64URL::new(),
                    parent_id: None,
                    fractional_index: "".to_string(),
                    doc: "".to_string(),
                    text: text.to_string(),
                    links: Links::new(),
                    path: None,
                    hidden: false,
                    created_at: now,
                    updated_at: now,
                }
            };

            let outlines = [
                outline("text"),
                outline("conflicting"),
                outline("conflicting"),
                outline("editing"),
                outline(""),
            ];

            for o in outlines.iter() {
                upsert_outline(app_handle, pot.id, o, vec![]).await.unwrap();
            }

            let result =
                fetch_conflicting_outline_ids(app_handle.clone(), outlines[4].id, None, "editing")
                    .await
                    .unwrap();

            assert_eq!(result.len(), 3);
        })
    }
}
