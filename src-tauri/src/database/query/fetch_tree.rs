use super::fetch_breadcrumbs;
use super::fetch_breadcrumbs::Breadcrumb;
use crate::database::table::CardsTable;
use crate::database::table::OutlinesTable;
use crate::database::types::Base64String;
use anyhow::anyhow;
use sqlx::query_as;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_tree<R: Runtime>(
    app_handle: AppHandle<R>,
    id: Base64String,
    depth: Option<u32>,
) -> anyhow::Result<(Vec<OutlinesTable>, Vec<CardsTable>, Vec<Breadcrumb>)> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    let outlines = fetch_outline_tree(&id, depth, pool).await?;

    let root = outlines
        .iter()
        .find(|o| o.id == id)
        .ok_or(anyhow!("failed to find root outline"))?;

    let breadcrumbs = match root.parent_id.0 {
        Some(ref id) => fetch_breadcrumbs(vec![id], pool).await?,
        None => vec![],
    };

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

        let mut query_builder = query_as::<_, CardsTable>(&query);

        for outline in outlines.iter() {
            query_builder = query_builder.bind(&outline.id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards, breadcrumbs))
}

async fn fetch_outline_tree(
    id: &Base64String,
    depth: Option<u32>,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<OutlinesTable>> {
    match depth {
        Some(depth) => sqlx::query_as!(OutlinesTable, r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, author, pot_id, parent_id, fractional_index, text, last_materialized_hash, created_at,
                        updated_at, is_deleted, 0 AS depth
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.author, child.pot_id, child.parent_id, child.fractional_index, child.text,
                        child.last_materialized_hash, child.created_at, child.updated_at, child.is_deleted,
                        parent.depth + 1 AS depth
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false AND depth <= ?
                )
                SELECT
                    id, author, pot_id, parent_id, fractional_index, text, last_materialized_hash, created_at,
                    updated_at, is_deleted
                FROM outline_tree;
                "#, id, depth)
                .fetch_all(pool).await,
        None => sqlx::query_as!(OutlinesTable, r#"
                WITH RECURSIVE outline_tree AS (
                    SELECT
                        id, author, pot_id, parent_id, fractional_index, text, last_materialized_hash, created_at,
                        updated_at, is_deleted
                    FROM outlines
                    WHERE id = ? AND is_deleted = false
                    UNION ALL
                    SELECT
                        child.id, child.author, child.pot_id, child.parent_id, child.fractional_index, child.text,
                        child.last_materialized_hash, child.created_at, child.updated_at, child.is_deleted
                    FROM outline_tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false
                )
                SELECT * FROM outline_tree;
                "#, id)
                .fetch_all(pool).await
    }
    .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_tree;
    use crate::database::types::NullableBase64String;
    use crate::test::*;

    #[test]
    fn test_fetch_tree() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let outline = create_tree(app_handle, NullableBase64String::none(), 2, 0).await;

        let (outlines, cards, breadcrumbs) = fetch_tree(app_handle.clone(), outline.id, None)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(cards.len(), 3);
        assert_eq!(breadcrumbs.len(), 0);
    }
}
