use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use crate::{database::query::fetch, types::model::LinkCount};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
pub async fn fetch_relation_count<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<UUIDv7Base64URL>,
    card_ids: Vec<UUIDv7Base64URL>,
    count_children: bool,
) -> eyre::Result<Vec<LinkCount>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    if count_children {
        fetch::recursive_relation_count(pool, &outline_ids, &card_ids).await
    } else {
        fetch::relation_count(pool, &outline_ids, &card_ids).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::create_version::test::create_version;
    use crate::commands::upsert_card::test::upsert_card;
    use crate::commands::upsert_outline::test::upsert_outline;
    use crate::database::test::create_mock_user_and_pot;
    use crate::test::run_in_mock_app;
    use crate::types::model::{Card, Outline, Quote};
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_relation_count() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let (_, pot) = create_mock_user_and_pot(app_handle.clone()).await;
            test_count(app_handle, pot.id).await;
        });
    }

    #[test]
    fn test_fetch_relation_count_recursively() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let (_, pot) = create_mock_user_and_pot(app_handle.clone()).await;
            test_count_recursively(app_handle, pot.id).await;
        });
    }

    async fn test_count(app_handle: &AppHandle<MockRuntime>, pot_id: UUIDv7Base64URL) {
        let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();

        // outline1, card1 → outline2
        // card1 → card2
        let ((o1, o2), (c1, c2)) = insert_test_data_for_test_count(app_handle, pool, pot_id).await;

        let mut result = fetch_relation_count(app_handle.clone(), vec![o1.id], vec![], false)
            .await
            .unwrap();
        assert_eq!(result.pop().unwrap().forward, 1);

        let mut result = fetch_relation_count(app_handle.clone(), vec![o2.id], vec![], false)
            .await
            .unwrap();
        assert_eq!(result.pop().unwrap().back, 2);

        let mut result = fetch_relation_count(app_handle.clone(), vec![], vec![c1.id], false)
            .await
            .unwrap();
        assert_eq!(result.pop().unwrap().forward, 2);

        let mut result = fetch_relation_count(app_handle.clone(), vec![], vec![c2.id], false)
            .await
            .unwrap();
        assert_eq!(result.pop().unwrap().back, 1);
    }

    async fn test_count_recursively(app_handle: &AppHandle<MockRuntime>, pot_id: UUIDv7Base64URL) {
        let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();

        // o3 → o1 → o6         back: 4, forward: 3
        // o4 → |_o2 → o7       back: 3, forward: 2
        // c2 ↗→ \c1 → o8    back: 2, forward: 1

        let ((o1, o2, _, _, _, _, _), (c1, _)) =
            insert_test_data_for_test_count_recursively(app_handle, pool, pot_id).await;

        let result =
            fetch_relation_count(app_handle.clone(), vec![o1.id, o2.id], vec![c1.id], true)
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
        pot_id: UUIDv7Base64URL,
    ) -> ((Outline, Outline), (Card, Card)) {
        let version_id = UUIDv7Base64URL::new();
        create_version(app_handle.clone(), pot_id, version_id)
            .await
            .unwrap();

        let o1 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o1, vec![])
            .await
            .unwrap();

        let o2 = Outline::new(Some(o1.id));
        upsert_outline(app_handle, pot_id, &o2, vec![])
            .await
            .unwrap();

        let c2 = Card::new(o2.id, None);
        upsert_card(app_handle, pot_id, &c2, vec![]).await.unwrap();

        let c1 = Card::new(
            o1.id,
            Some(Quote {
                id: c2.id,
                version_id,
            }),
        );
        upsert_card(app_handle, pot_id, &c1, vec![]).await.unwrap();

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
        pot_id: UUIDv7Base64URL,
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
        (Card, Card),
    ) {
        let version_id = UUIDv7Base64URL::new();
        create_version(app_handle.clone(), pot_id, version_id)
            .await
            .unwrap();

        let o1 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o1, vec![])
            .await
            .unwrap();

        let o2 = Outline::new(Some(o1.id));
        upsert_outline(app_handle, pot_id, &o2, vec![])
            .await
            .unwrap();

        let o3 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o3, vec![])
            .await
            .unwrap();

        let o4 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o4, vec![])
            .await
            .unwrap();

        let o5 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o5, vec![])
            .await
            .unwrap();

        let o6 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o6, vec![])
            .await
            .unwrap();

        let o7 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o7, vec![])
            .await
            .unwrap();

        let o8 = Outline::new(None);
        upsert_outline(app_handle, pot_id, &o8, vec![])
            .await
            .unwrap();

        let c1 = Card::new(o2.id, None);
        upsert_card(app_handle, pot_id, &c1, vec![]).await.unwrap();

        let c2 = Card::new(
            o3.id,
            Some(Quote {
                id: c1.id,
                version_id,
            }),
        );
        upsert_card(app_handle, pot_id, &c2, vec![]).await.unwrap();

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
            o6.id,
            o2.id,
            o7.id
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO card_links (id_from, id_to)
                VALUES (?, ?), (?, ?);
            "#,
            c1.id,
            o8.id,
            c2.id,
            o2.id,
        )
        .execute(pool)
        .await
        .unwrap();

        ((o1, o2, o3, o4, o5, o6, o7), (c1, c2))
    }
}
