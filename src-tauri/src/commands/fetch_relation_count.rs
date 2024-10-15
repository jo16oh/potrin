use crate::database::query::count_relation;
use crate::types::util::Base64;
use crate::{database::query::count_relation_recursively, types::model::LinkCount};
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

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
        count_relation_recursively(pool, &outline_ids, &card_ids).await
    } else {
        count_relation(pool, &outline_ids, &card_ids).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::query;
    use crate::database::test::{create_mock_user_and_pot, insert_quote_without_versioning};
    use crate::test::run_in_mock_app;
    use crate::types::model::{Card, Outline};
    use crate::types::state::AppState;
    use crate::types::util::NullableBase64;
    use tauri::async_runtime::RwLock;
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

        let mut result = count_relation(pool, &[o1.id], &[]).await.unwrap();
        assert_eq!(result.pop().unwrap().forward, 1);

        let mut result = count_relation(pool, &[o2.id], &[]).await.unwrap();
        assert_eq!(result.pop().unwrap().back, 2);

        let mut result = count_relation(pool, &[], &[c1.id]).await.unwrap();
        assert_eq!(result.pop().unwrap().forward, 2);

        let mut result = count_relation(pool, &[], &[c2.id]).await.unwrap();
        assert_eq!(result.pop().unwrap().back, 1);
    }

    async fn test_count_recursively(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        // o3 → o1 → o6         back: 4, forward: 3
        // o4 → |_o2 → o7       back: 3, forward: 2
        // o5, c2 → \c1 → o8    back: 2, forward: 1

        let ((o1, o2, _, _, _, _, _), (c1, _, _)) =
            insert_test_data_for_test_count_recursively(app_handle, pool).await;

        let result =
            count_relation_recursively(pool, &[o1.id.clone(), o2.id.clone()], &[c1.id.clone()])
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
        let lock = app_handle.state::<RwLock<AppState>>().inner();

        let app_state = lock.read().await;
        let pot_id = &app_state
            .pot
            .as_ref()
            .ok_or(anyhow!("pot state is not set"))
            .unwrap()
            .id;

        let o1 = Outline::new(None);
        query::insert_outline(pool, &o1, pot_id).await.unwrap();

        let o2 = Outline::new(None);
        query::insert_outline(pool, &o2, pot_id).await.unwrap();

        let c1 = Card::new(o1.id.clone(), None);
        query::insert_card(pool, &c1).await.unwrap();

        let c2 = Card::new(o2.id.clone(), None);
        query::insert_card(pool, &c2).await.unwrap();

        insert_quote_without_versioning(app_handle.clone(), &c1.id, &c2.id)
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
        let lock = app_handle.state::<RwLock<AppState>>().inner();

        let app_state = lock.read().await;
        let pot_id = &app_state
            .pot
            .as_ref()
            .ok_or(anyhow!("pot state is not set"))
            .unwrap()
            .id;

        let o1 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o1, pot_id).await.unwrap();

        let o2 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::from(o1.id.clone()),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o2, pot_id).await.unwrap();

        let o3 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o3, pot_id).await.unwrap();

        let o4 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o4, pot_id).await.unwrap();

        let o5 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o5, pot_id).await.unwrap();
        let o6 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o6, pot_id).await.unwrap();

        let o7 = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::none(),
            fractional_index: String::new(),
            text: None,
        };
        query::insert_outline(pool, &o7, pot_id).await.unwrap();

        let c1 = Card::new(o2.id.clone(), None);
        query::insert_card(pool, &c1).await.unwrap();

        let c2 = Card::new(o3.id.clone(), None);
        query::insert_card(pool, &c2).await.unwrap();

        let c3 = Card::new(o4.id.clone(), None);
        query::insert_card(pool, &c3).await.unwrap();

        insert_quote_without_versioning(app_handle.clone(), &c2.id, &c1.id)
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

        ((o1, o2, o3, o4, o5, o6, o7), (c1, c2, c3))
    }
}
