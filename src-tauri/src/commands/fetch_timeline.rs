use crate::database::query::{fetch_cards_by_created_at, fetch_links, fetch_outlines_by_id};
use crate::types::model::{Card, Link, Outline};
use crate::types::util::Base64;
use crate::utils::get_state;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

// #[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
// #[serde(rename_all = "camelCase")]
// pub enum TlOption {
//     CreatedAt,
//     UpdatedAt,
//     Both,
// }

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    from: DateTime<Utc>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>, Vec<Link>)> {
    let to = from + Duration::days(1);

    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let cards = fetch_cards_by_created_at(pool, from, to).await?;
    let card_ids: Vec<&Base64> = cards.iter().map(|c| &c.id).collect();
    let outline_ids: Vec<&Base64> = cards.iter().map(|c| &c.outline_id).collect();
    let outlines = fetch_outlines_by_id(pool, &outline_ids).await?;
    let links = fetch_links(pool, &outline_ids, &card_ids).await?;

    Ok((outlines, cards, links))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_mock_user_and_pot;
    use crate::database::test::create_tree;
    use crate::test::run_in_mock_app;
    use chrono::Duration;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_timeline() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        // let now = Utc::now();
        create_tree(app_handle, None, 2, 0).await;

        // let (outlines, cards) = fetch_timeline(app_handle.clone(), now, TlOption::Both)
        //     .await
        //     .unwrap();
        //
        // assert_eq!(outlines.len(), 3);
        // assert_eq!(cards.len(), 3);
        //
        // let now = Utc::now() - Duration::days(2);
        // let (outlines, cards) = fetch_timeline(app_handle.clone(), now, TlOption::Both)
        //     .await
        //     .unwrap();
        //
        // assert_eq!(outlines.len(), 0);
        // assert_eq!(cards.len(), 0);

        let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();

        let time = (Utc::now() + Duration::days(3)).timestamp_millis();
        sqlx::query!(
            r#"
                UPDATE outlines
                SET updated_at = ?;
            "#,
            time
        )
        .execute(pool)
        .await
        .unwrap();

        let time = (Utc::now() + Duration::days(3)).timestamp_millis();
        sqlx::query!(
            r#"
                UPDATE cards
                SET updated_at = ?;
            "#,
            time
        )
        .execute(pool)
        .await
        .unwrap();

        // let (outlines, cards) = fetch_timeline(
        //     app_handle.clone(),
        //     Utc::now() - Duration::minutes(1),
        //     TlOption::Both,
        // )
        // .await
        // .unwrap();
        //
        // assert_eq!(outlines.len(), 3);
        // assert_eq!(cards.len(), 3);
        //
        // let (outlines, cards) = fetch_timeline(
        //     app_handle.clone(),
        //     Utc::now() - Duration::minutes(1),
        //     TlOption::CreatedAt,
        // )
        // .await
        // .unwrap();
        //
        // assert_eq!(outlines.len(), 3);
        // assert_eq!(cards.len(), 3);
        //
        // let (outlines, cards) = fetch_timeline(
        //     app_handle.clone(),
        //     Utc::now() - Duration::minutes(1),
        //     TlOption::UpdatedAt,
        // )
        // .await
        // .unwrap();
        //
        // assert_eq!(outlines.len(), 0);
        // assert_eq!(cards.len(), 0);

        let time = (Utc::now() + Duration::days(3)).timestamp_millis();
        sqlx::query!(
            r#"
                UPDATE outlines
                SET created_at = ?;
            "#,
            time
        )
        .execute(pool)
        .await
        .unwrap();

        let time = (Utc::now() + Duration::days(3)).timestamp_millis();
        sqlx::query!(
            r#"
                UPDATE cards
                SET created_at = ?;
            "#,
            time
        )
        .execute(pool)
        .await
        .unwrap();

        // let (outlines, cards) = fetch_timeline(
        //     app_handle.clone(),
        //     Utc::now() - Duration::minutes(1),
        //     TlOption::Both,
        // )
        // .await
        // .unwrap();
        //
        // assert_eq!(outlines.len(), 0);
        // assert_eq!(cards.len(), 0);

        let (outlines, cards, _links) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            // TlOption::CreatedAt,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(cards.len(), 0);
    }
}
