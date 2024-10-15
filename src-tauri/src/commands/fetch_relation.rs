use crate::database::query::fetch_descendant_ids;
use crate::database::query::{fetch_relation_back, fetch_relation_forward};
use crate::types::model::Card;
use crate::types::model::Outline;
use crate::types::util::Base64;
use crate::utils::get_state;
use serde::Deserialize;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RelationOption {
    direction: Direction,
    include_children: Option<IncludeChildrenOption>,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
enum Direction {
    Back,
    Forward,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
struct IncludeChildrenOption {
    include_cards: bool,
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
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let (outline_ids, card_ids) = match option.include_children {
        Some(opt) => fetch_descendant_ids(pool, &outline_ids, opt.include_cards).await?,
        None => (outline_ids, card_ids),
    };

    match option.direction {
        Direction::Back => fetch_relation_back(pool, &outline_ids, &card_ids).await,
        Direction::Forward => fetch_relation_forward(pool, &outline_ids, &card_ids).await,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::query;
    use crate::database::test::create_mock_user_and_pot;
    use crate::database::test::create_tree;
    use crate::database::test::insert_quote_without_versioning;
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
        let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();

        let r1 = create_tree(app_handle, None, 3, 0).await;
        let r2 = create_tree(app_handle, None, 3, 0).await;

        let c1 = Card::new(r1.id.clone(), None);
        query::insert_card(pool, &c1).await.unwrap();

        let c2 = Card::new(r2.id.clone(), None);
        query::insert_card(pool, &c2).await.unwrap();

        let c3 = Card::new(r1.id.clone(), None);
        query::insert_card(pool, &c3).await.unwrap();

        sqlx::query!(
            r#"
                INSERT INTO outline_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            r1.id,
            r2.id
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
            r2.id
        )
        .execute(pool)
        .await
        .unwrap();

        insert_quote_without_versioning(app_handle.clone(), &c1.id, &c2.id)
            .await
            .unwrap();
        insert_quote_without_versioning(app_handle.clone(), &c2.id, &c3.id)
            .await
            .unwrap();

        let (outlines, cards) = fetch_relation(
            app_handle.clone(),
            vec![r2.id],
            vec![],
            RelationOption {
                direction: Direction::Back,
                include_children: Some(IncludeChildrenOption {
                    include_cards: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(cards.len(), 2);

        let (outlines, cards) = fetch_relation(
            app_handle.clone(),
            vec![r1.id],
            vec![],
            RelationOption {
                direction: Direction::Forward,
                include_children: Some(IncludeChildrenOption {
                    include_cards: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(cards.len(), 2);
    }
}
