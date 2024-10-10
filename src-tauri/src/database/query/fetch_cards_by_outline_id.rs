use crate::types::{model::Card, util::Base64};
use anyhow::anyhow;
use sqlx::{query_as, SqlitePool};

pub async fn fetch_cards_by_outline_id(
    pool: &SqlitePool,
    outline_ids: &[&Base64],
) -> anyhow::Result<Vec<Card>> {
    let query = format!(
        r#"
            WITH c1 AS (
                SELECT id, outline_id, fractional_index, text, quote
                FROM cards AS c1
                WHERE outline_id IN ({}) AND is_deleted = false
            )
            SELECT *
            FROM c1
            UNION 
            SELECT id, outline_id, fractional_index, text, quote 
            FROM cards 
            WHERE id IN ((SELECT quote FROM c1)) AND is_deleted = false;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = query_as::<_, Card>(&query);

    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        database::{
            query::{insert_card, insert_outline},
            test::create_mock_user_and_pot,
        },
        run_in_mock_app,
        types::{model::Outline, state::AppState},
    };
    use std::sync::RwLock;
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

        let pot_id = {
            let app_state = lock.read().unwrap();
            let pot = app_state.pot.as_ref().unwrap();
            pot.id.clone()
        };

        let o1 = Outline::new(None);
        let o2 = Outline::new(None);
        insert_outline(pool, &o1, &pot_id).await.unwrap();
        insert_outline(pool, &o2, &pot_id).await.unwrap();
        let c1 = Card::new(o1.id.clone(), None);
        let c2 = Card::new(o2.id.clone(), Some(c1.id.clone()));
        insert_card(pool, &c1).await.unwrap();
        insert_card(pool, &c2).await.unwrap();

        let outline_ids = vec![&o2.id];
        let result = fetch_cards_by_outline_id(pool, &outline_ids).await.unwrap();

        assert_eq!(result.len(), 2);
    }
}
