use crate::types::util::Base64;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct LinkCount {
    id: Base64,
    back: i64,
    forward: i64,
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_relation_count<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
    count_children: bool,
) -> anyhow::Result<Vec<LinkCount>> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))
        .unwrap()
        .inner();

    if count_children {
        count_relation_recursively(pool, outline_ids, card_ids).await
    } else {
        count_relation(pool, outline_ids, card_ids).await
    }
}

async fn count_relation(
    pool: &SqlitePool,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
) -> anyhow::Result<Vec<LinkCount>> {
    let query = format!(
        r#"
            SELECT
                id,
                (
                    (
                        SELECT COUNT(*)
                        FROM outline_links
                        WHERE outline_links.id_to = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM card_links
                        WHERE card_links.id_to = this.id
                    )
                ) AS back,
                (
                    SELECT COUNT(*)
                    FROM outline_links
                    WHERE outline_links.id_from = this.id
                ) AS forward
            FROM outlines AS this
            WHERE id IN ({})
            UNION ALL
            SELECT
                id,
                (
                    SELECT COUNT(*)
                    FROM card_quotes
                    WHERE card_quotes.id_to = this.id
                ) AS back,
                (
                    (
                        SELECT COUNT(*)
                        FROM card_links
                        WHERE card_links.id_from = this.id
                    )
                    +
                    (
                        SELECT COUNT(*)
                        FROM card_quotes
                        WHERE card_quotes.id_from = this.id
                    )
                ) AS forward
            FROM cards AS this
            WHERE id IN ({});
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
            .join(", "),
    );

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in card_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}

async fn count_relation_recursively(
    pool: &SqlitePool,
    outline_ids: Vec<Base64>,
    card_ids: Vec<Base64>,
) -> anyhow::Result<Vec<LinkCount>> {
    let query = format!(
        r#"
                WITH RECURSIVE tree AS (
                    SELECT id, id AS root_id
                    FROM outlines
                    WHERE id IN ({}) AND is_deleted = false
                    UNION ALL
                    SELECT child.id, parent.root_id AS root_id
                    FROM tree AS parent
                    JOIN outlines AS child ON parent.id = child.parent_id
                    WHERE child.is_deleted = false
                ),
                tree_cards AS (
                    SELECT cards.id, tree.root_id
                    FROM cards
                    INNER JOIN tree ON cards.outline_id = tree.id
                )
                SELECT
                    id,
                    (
                        (
                            SELECT COUNT(*)
                            FROM outline_links
                            WHERE outline_links.id_to IN ((
                                SELECT id
                                FROM tree
                                WHERE tree.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_to IN ((
                                SELECT id
                                FROM tree
                                WHERE tree.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM card_quotes
                            WHERE card_quotes.id_to IN ((
                                SELECT id
                                FROM tree_cards
                                WHERE tree_cards.root_id = this.id
                            ))
                        )
                    ) AS back,
                    (
                        SELECT COUNT(*)
                        FROM outline_links
                        WHERE outline_links.id_from IN ((
                            SELECT id
                            FROM tree
                            WHERE tree.root_id = this.id
                        ))
                    )
                    +
                    (
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_from IN ((
                                SELECT id
                                FROM tree_cards
                                WHERE tree_cards.root_id = this.id
                            ))
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM card_quotes
                            WHERE card_quotes.id_from IN ((
                                SELECT id
                                FROM tree_cards
                                WHERE tree_cards.root_id = this.id
                            ))
                        )
                    ) AS forward
                FROM tree AS this
                WHERE id = root_id
                UNION ALL
                SELECT DISTINCT
                    id,
                    (
                        SELECT COUNT(*)
                        FROM card_quotes
                        WHERE card_quotes.id_to = this.id
                    ) AS back,
                    (
                        (
                            SELECT COUNT(*)
                            FROM card_links
                            WHERE card_links.id_from = this.id
                        )
                        +
                        (
                            SELECT COUNT(*)
                            FROM card_quotes
                            WHERE card_quotes.id_from = this.id
                        )
                    ) AS forward
                FROM tree_cards AS this
                WHERE id IN ({});
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
            .join(", "),
    );

    let mut query_builder = sqlx::query_as::<_, LinkCount>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    for id in card_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::{insert_card, insert_outline};
    use crate::database::test::create_mock_user_and_pot;
    use crate::test::run_in_mock_app;
    use crate::types::model::{Card, CardYUpdate, Outline, OutlineYUpdate};
    use crate::types::util::NullableBase64;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_relation_count() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test_count(app_handle).await;
        });
    }

    #[test]
    fn test_fetch_relation_count_recursively() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test_count_recursively(app_handle).await;
        });
    }

    async fn test_count(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        // outline1, card1 → outline2
        // card1 → card2
        let ((o1, o2), (c1, c2)) = insert_test_data_for_test_count(app_handle, pool).await;

        let mut result = count_relation(pool, vec![o1.id], vec![]).await.unwrap();
        assert_eq!(result.pop().unwrap().forward, 1);

        let mut result = count_relation(pool, vec![o2.id], vec![]).await.unwrap();
        assert_eq!(result.pop().unwrap().back, 2);

        let mut result = count_relation(pool, vec![], vec![c1.id]).await.unwrap();
        assert_eq!(result.pop().unwrap().forward, 2);

        let mut result = count_relation(pool, vec![], vec![c2.id]).await.unwrap();
        assert_eq!(result.pop().unwrap().back, 1);
    }

    async fn test_count_recursively(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        // o3 → o1 → o6         back: 4, forward: 3
        // o4 → |_o2 → o7       back: 3, forward: 2
        // o5, c2 → \c1 → o8    back: 2, forward: 1

        let ((o1, o2, _, _, _, _, _), (c1, _, _)) =
            insert_test_data_for_test_count_recursively(app_handle, pool).await;

        let result = count_relation_recursively(
            pool,
            vec![o1.id.clone(), o2.id.clone()],
            vec![c1.id.clone()],
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 3);

        for r in result.iter() {
            if r.id == o1.id {
                assert_eq!(r.back, 4);
                assert_eq!(r.forward, 3);
            } else if r.id == o2.id {
                assert_eq!(r.back, 3);
                assert_eq!(r.forward, 2);
            } else if r.id == c1.id {
                assert_eq!(r.back, 1);
                assert_eq!(r.forward, 1);
            } else {
                panic!();
            }
        }
    }

    async fn insert_test_data_for_test_count(
        app_handle: &AppHandle<MockRuntime>,
        pool: &SqlitePool,
    ) -> ((Outline, Outline), (Card, Card)) {
        let o1 = insert_outline(
            app_handle.clone(),
            Outline::new(None),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        let o2 = insert_outline(
            app_handle.clone(),
            Outline::new(None),
            vec![OutlineYUpdate::new()],
        )
        .await
        .unwrap();

        let c1 = insert_card(
            app_handle.clone(),
            Card::new(o1.id.clone()),
            vec![CardYUpdate::new()],
        )
        .await
        .unwrap();

        let c2 = insert_card(
            app_handle.clone(),
            Card::new(o2.id.clone()),
            vec![CardYUpdate::new()],
        )
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO outline_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            o1.id,
            o2.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            c1.id,
            o2.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_quotes (id_from, id_to)
                VALUES (?, ?);
            "#,
            c1.id,
            c2.id
        )
        .execute(pool)
        .await
        .unwrap();

        ((o1, o2), (c1, c2))
    }

    async fn insert_test_data_for_test_count_recursively(
        app_handle: &AppHandle<MockRuntime>,
        pool: &SqlitePool,
    ) -> (
        (
            Outline,
            Outline,
            Outline,
            Outline,
            Outline,
            Outline,
            Outline,
        ),
        (Card, Card, Card),
    ) {
        let o1 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o1.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let o2 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::from(o1.id.clone()),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o2.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let o3 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o3.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let o4 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o4.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let o5 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o5.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();
        let o6 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o6.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let o7 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        insert_outline(app_handle.clone(), o7.clone(), vec![OutlineYUpdate::new()])
            .await
            .unwrap();

        let c1 = Card {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            outline_id: o2.id.clone(),
            fractional_index: String::new(),
            text: String::new(),
        };
        insert_card(app_handle.clone(), c1.clone(), vec![CardYUpdate::new()])
            .await
            .unwrap();

        let c2 = Card {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            outline_id: o3.id.clone(),
            fractional_index: String::new(),
            text: String::new(),
        };
        insert_card(app_handle.clone(), c2.clone(), vec![CardYUpdate::new()])
            .await
            .unwrap();

        let c3 = Card {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            outline_id: o4.id.clone(),
            fractional_index: String::new(),
            text: String::new(),
        };
        insert_card(app_handle.clone(), c3.clone(), vec![CardYUpdate::new()])
            .await
            .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO outline_links (id_from, id_to)
                VALUES (?, ?), (?, ?), (?, ?), (?, ?);
            "#,
            o3.id,
            o1.id,
            o4.id,
            o2.id,
            o1.id,
            o5.id,
            o2.id,
            o6.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_links (id_from, id_to)
                VALUES (?, ?), (?, ?);
            "#,
            c2.id,
            o2.id,
            c1.id,
            o7.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_quotes (id_from, id_to)
                VALUES (?, ?);
            "#,
            c3.id,
            c1.id
        )
        .execute(pool)
        .await
        .unwrap();

        ((o1, o2, o3, o4, o5, o6, o7), (c1, c2, c3))
    }
}
