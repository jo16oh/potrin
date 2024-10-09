use crate::database::table::Card;
use crate::database::table::Outline;
use crate::database::types::Base64;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct RelationOption {
    direction: Direction,
    include_children: Option<IncludeChildrenOption>,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
enum Direction {
    Back,
    Forward,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
struct IncludeChildrenOption {
    card: bool,
}

#[derive(FromRow)]
struct QueryResult {
    pub id: Base64,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_relation<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
    option: RelationOption,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))
        .unwrap()
        .inner();

    let (outline_ids, card_ids) = match option.include_children {
        Some(opt) => {
            let outline_ids = {
                let query = format!(
                    r#"
                        WITH RECURSIVE outline_tree AS (
                            SELECT id
                            FROM outlines
                            WHERE id = {} AND is_deleted = false
                            UNION ALL
                            SELECT child.id
                            FROM outline_tree AS parent
                            JOIN outlines AS child ON parent.id = child.parent_id
                            WHERE child.is_deleted = false
                        )
                        SELECT id FROM outline_tree;
                    "#,
                    outline_ids
                        .iter()
                        .map(|_| "?".to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );

                let mut query_builder = sqlx::query_as::<_, QueryResult>(&query);

                for id in outline_ids.iter() {
                    query_builder = query_builder.bind(id);
                }

                query_builder
                    .fetch_all(pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|r| r.id)
                    .collect::<Vec<Base64>>()
            };

            let card_ids = if opt.card {
                let query = format!(
                    r#"
                        SELECT id FROM cards WHERE outline_id IN ({}) AND is_deleted = false;
                    "#,
                    outline_ids
                        .iter()
                        .map(|_| "?".to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );

                let mut query_builder = sqlx::query_as::<_, QueryResult>(&query);

                for id in outline_ids.iter() {
                    query_builder = query_builder.bind(id);
                }

                query_builder
                    .fetch_all(pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|r| r.id)
                    .collect::<Vec<Base64>>()
            } else {
                card_ids
            };

            (outline_ids, card_ids)
        }
        None => (outline_ids, card_ids),
    };

    match option.direction {
        Direction::Back => fetch_relation_back(pool, outline_ids, card_ids).await,
        Direction::Forward => fetch_relation_forward(pool, outline_ids, card_ids).await,
    }
}

async fn fetch_relation_back(
    pool: &SqlitePool,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    id, parent_id, fractional_index, text
                FROM outline_links
                INNER JOIN outlines ON outline_links.id_from = outlines.id
                WHERE outlines.is_deleted = false AND id_to IN ({});
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    let cards: Vec<Card> = {
        let query = format!(
            r#"
                SELECT
                    id, outline_id, fractional_index, text
                FROM cards
                INNER JOIN card_links ON card_links.id_from = cards.id
                INNER JOIN card_quotes ON card_links.id_from = cards.id
                WHERE (card_links.id_to IN ({}) OR card_quotes.id_to IN ({})) AND cards.is_deleted = false;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", "),
            card_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Card>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}

async fn fetch_relation_forward(
    pool: &SqlitePool,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
) -> anyhow::Result<(Vec<Outline>, Vec<Card>)> {
    let outlines = {
        let query = format!(
            r#"
                SELECT
                    id, parent_id, fractional_index, text
                FROM outlines
                INNER JOIN outline_links ON outline_links.id_to = outlines.id
                INNER JOIN card_links ON card_links.id_to = outlines.id
                WHERE outline_links.id_from IN ({}) AND outlines.is_deleted = false;
            "#,
            outline_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Outline>(&query);

        for id in outline_ids.iter() {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    let cards: Vec<Card> = {
        let query = format!(
            r#"
                    SELECT
                        id, outline_id, fractional_index, text
                    FROM card_quotes
                    INNER JOIN cards ON card_quotes.id_to = cards.id
                    WHERE cards.is_deleted = false AND card_quotes.id_from IN ({});
            "#,
            card_ids
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Card>(&query);

        for id in card_ids {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(pool).await?
    };

    Ok((outlines, cards))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::insert_card;
    use crate::database::table::{Card, CardYUpdate};
    use crate::database::test::create_mock_user_and_pot;
    use crate::database::test::create_tree;
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_relation() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        let root = create_tree(app_handle, None, 3, 0).await;
        let outline = create_tree(app_handle, None, 3, 0).await;
        let card1 = insert_card(
            app_handle.clone(),
            Card::new(root.id.clone()),
            vec![CardYUpdate::new()],
        )
        .await
        .unwrap();
        let card2 = insert_card(
            app_handle.clone(),
            Card::new(outline.id.clone()),
            vec![CardYUpdate::new()],
        )
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO outline_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            root.id,
            outline.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            card1.id,
            outline.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_quotes (id_from, id_to)
                VALUES (?, ?);
            "#,
            card1.id,
            card2.id
        )
        .execute(pool)
        .await
        .unwrap();

        let (outlines, cards) = fetch_relation(
            app_handle.clone(),
            vec![outline.id],
            vec![],
            RelationOption {
                direction: Direction::Back,
                include_children: Some(IncludeChildrenOption { card: true }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(cards.len(), 1);

        let (outlines, cards) = fetch_relation(
            app_handle.clone(),
            vec![root.id],
            vec![],
            RelationOption {
                direction: Direction::Forward,
                include_children: Some(IncludeChildrenOption { card: true }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(cards.len(), 1);
    }
}
