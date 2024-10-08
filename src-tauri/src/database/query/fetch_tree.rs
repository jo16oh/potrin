use crate::database::table::Card;
use crate::database::table::Outline;
use crate::database::types::Base64;
use anyhow::anyhow;
use sqlx::query_as;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_tree<R: Runtime>(
    app_handle: AppHandle<R>,
    id: Base64,
    depth: Option<u32>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let outlines = fetch_outline_tree(&id, depth, pool).await?;

    let cards = {
        let query = format!(
            r#"
                SELECT * FROM cards WHERE outline_id IN ({}) AND is_deleted = false;
            "#,
            outlines
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = query_as::<_, Card>(&query);

        for outline in outlines.iter() {
            query_builder = query_builder.bind(&outline.id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}

async fn fetch_outline_tree(
    id: &Base64,
    depth: Option<u32>,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<Outline>> {
    match depth {
        Some(depth) => {
            sqlx::query_as!(
                Outline,
                r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, parent_id, fractional_index, text, 0 AS depth
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.parent_id, child.fractional_index, child.text,
                        parent.depth + 1 AS depth
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false AND depth <= ?
                )
                SELECT
                    id, parent_id, fractional_index, text
                FROM outline_tree;
                "#,
                id,
                depth
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Outline,
                r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, parent_id, fractional_index, text
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.parent_id, child.fractional_index, child.text
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false
                )
                SELECT * FROM outline_tree;
                "#,
                id
            )
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::{create_mock_user_and_pot, create_tree};
    use crate::test::*;

    #[test]
    fn test_fetch_tree() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let outline = create_tree(app_handle, None, 2, 0).await;

        let (outlines, cards) = fetch_tree(app_handle.clone(), outline.id, None)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);
    }
}
