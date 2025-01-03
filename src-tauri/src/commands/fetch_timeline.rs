use crate::database::query::fetch;
use crate::types::model::{Card, Outline};
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    from: DateTime<Utc>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let to = from + Duration::days(1);

    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let cards = fetch::cards_by_created_at(pool, from, to).await?;
    let outline_ids: Vec<UUIDv7Base64URL> = cards.iter().map(|c| c.outline_id).collect();
    let outlines = fetch::outlines_by_id(pool, &outline_ids).await?;

    Ok((outlines, cards))
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::database::test::create_mock_user_and_pot;
//     use crate::database::test::create_tree;
//     use crate::test::run_in_mock_app;
//     use chrono::Duration;
//     use tauri::test::MockRuntime;
//
//     #[test]
//     fn test_fetch_timeline() {
//         run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
//             let (_, pot) = create_mock_user_and_pot(app_handle.clone()).await;
//             create_tree(app_handle, pot.id, None, 2, 0).await;
//             let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();
//
//             let time = (Utc::now() + Duration::days(3)).timestamp_millis();
//             sqlx::query!(
//                 r#"
//                 UPDATE outlines
//                 SET updated_at = ?;
//             "#,
//                 time
//             )
//             .execute(pool)
//             .await
//             .unwrap();
//
//             let time = (Utc::now() + Duration::days(3)).timestamp_millis();
//             sqlx::query!(
//                 r#"
//                 UPDATE cards
//                 SET updated_at = ?;
//             "#,
//                 time
//             )
//             .execute(pool)
//             .await
//             .unwrap();
//
//             let time = (Utc::now() + Duration::days(3)).timestamp_millis();
//             sqlx::query!(
//                 r#"
//                 UPDATE y_docs
//                 SET created_at = ?;
//             "#,
//                 time
//             )
//             .execute(pool)
//             .await
//             .unwrap();
//
//             let time = (Utc::now() + Duration::days(3)).timestamp_millis();
//             sqlx::query!(
//                 r#"
//                 UPDATE y_docs
//                 SET created_at = ?;
//                     "#,
//                 time
//             )
//             .execute(pool)
//             .await
//             .unwrap();
//
//             let (outlines, cards) = fetch_timeline(
//                 app_handle.clone(),
//                 Utc::now() - Duration::minutes(1),
//                 // TlOption::CreatedAt,
//             )
//             .await
//             .unwrap();
//
//             assert_eq!(outlines.len(), 0);
//             assert_eq!(cards.len(), 0);
//         });
//     }
// }
