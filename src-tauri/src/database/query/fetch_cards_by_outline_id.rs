use crate::types::{
    model::{Card, RawCard},
    util::Base64,
};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_cards_by_outline_id(
    pool: &SqlitePool,
    outline_ids: &[&Base64],
) -> anyhow::Result<Vec<Card>> {
    let query = format!(
        r#"
            WITH c1 AS (
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.text,
                    cards.version_id AS version_id,
                    quotes.quoted_card_id AS quoted_card_id,
                    quotes.version_id AS quote_version_id
                FROM cards
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE outline_id IN ({}) AND is_deleted = false
                UNION
                SELECT
                    cards.id, cards.outline_id, cards.fractional_index, cards.text,
                    cards.version_id AS version_id,
                    quotes.quoted_card_id AS quoted_card_id,
                    quotes.version_id AS quote_version_id
                FROM cards
                JOIN c1 ON cards.id = c1.quoted_card_id
                LEFT JOIN quotes ON cards.id = quotes.card_id
                WHERE is_deleted = false
            )
            SELECT *
            FROM c1;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = query_as::<_, RawCard>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .fetch_all(pool)
        .await
        .map(|raw_cards| raw_cards.into_iter().map(Card::from).collect())
        .map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        database::{
            query::{insert_card, insert_outline},
            test::{create_mock_user_and_pot, insert_quote_without_versioning},
        },
        run_in_mock_app,
        types::{model::Outline, state::AppState},
    };
    use tauri::async_runtime::RwLock;
    use tauri::{test::MockRuntime, AppHandle, Manager};

    #[test]
    fn test_fetch_cards_by_ids() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle.clone()).await;
        })
    }

    async fn test(app_handle: AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();
        let lock = app_handle.state::<RwLock<AppState>>().inner();

        let app_state = lock.read().await;
        let pot_id = &app_state
            .pot
            .as_ref()
            .ok_or(anyhow!("pot state is not set"))
            .unwrap()
            .id;

        let o1 = Outline::new(None);
        let o2 = Outline::new(None);
        insert_outline(pool, &o1, pot_id).await.unwrap();
        insert_outline(pool, &o2, pot_id).await.unwrap();
        let c1 = Card::new(o1.id.clone(), None);
        let c2 = Card::new(o2.id.clone(), None);
        insert_card(pool, &c1).await.unwrap();
        insert_card(pool, &c2).await.unwrap();

        insert_quote_without_versioning(app_handle.clone(), &c2.id, &c1.id)
            .await
            .unwrap();

        let outline_ids = vec![&o2.id];
        let result = fetch_cards_by_outline_id(pool, &outline_ids).await.unwrap();

        assert_eq!(result.len(), 2);
    }
}
