use crate::database::table::Card;
use crate::database::table::Outline;
use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub enum TlOption {
    CreatedAt,
    UpdatedAt,
    Both,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    from: DateTime<Utc>,
    option: TlOption,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let to = (from + Duration::days(1)).timestamp_millis();
    let from = from.timestamp_millis();

    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let cards = match option {
        TlOption::CreatedAt => {
            sqlx::query_as!(
                Card,
                r#"
                    SELECT id, outline_id, fractional_index, text
                    FROM cards
                    WHERE ? <= created_at AND created_at < ? AND is_deleted = false;
                "#,
                from,
                to,
            )
            .fetch_all(pool)
            .await
        }
        TlOption::UpdatedAt => {
            sqlx::query_as!(
                Card,
                r#"
                    SELECT id, outline_id, fractional_index, text
                    FROM cards
                    WHERE ? <= updated_at AND updated_at < ? AND is_deleted = false;
                "#,
                from,
                to,
            )
            .fetch_all(pool)
            .await
        }
        TlOption::Both => {
            sqlx::query_as!(
                Card,
                r#"
                    SELECT id, outline_id, fractional_index, text
                    FROM cards
                    WHERE
                        ((? <= updated_at AND updated_at < ?) OR (? <= created_at AND created_at < ?))
                        AND is_deleted = false;
                "#,
                from,
                to,
                from,
                to
            )
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| anyhow!(e.to_string()))?;

    let outlines = {
        let query = format!(
            r#"
                SELECT id, parent_id, fractional_index, text
                FROM outlines
                WHERE id IN ({}) AND is_deleted = false;
            "#,
            cards
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for card in cards.iter() {
            query_builder = query_builder.bind(&card.outline_id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_tree;
    use crate::test::*;
    use chrono::Duration;

    #[test]
    fn test_fetch_timeline() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let now = Utc::now();
        create_tree(app_handle, None, 2, 0).await;

        let (outlines, cards) = fetch_timeline(app_handle.clone(), now, TlOption::Both)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);

        let now = Utc::now() - Duration::days(2);
        let (outlines, cards) = fetch_timeline(app_handle.clone(), now, TlOption::Both)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(cards.len(), 0);

        let pool = app_handle.state::<SqlitePool>().inner();

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

        let (outlines, cards) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            TlOption::Both,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);

        let (outlines, cards) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            TlOption::CreatedAt,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);

        let (outlines, cards) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            TlOption::UpdatedAt,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(cards.len(), 0);

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

        let (outlines, cards) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            TlOption::Both,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(cards.len(), 0);

        let (outlines, cards) = fetch_timeline(
            app_handle.clone(),
            Utc::now() - Duration::minutes(1),
            TlOption::CreatedAt,
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(cards.len(), 0);
    }
}
