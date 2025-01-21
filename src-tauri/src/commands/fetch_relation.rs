use crate::database::query::fetch;
use crate::types::model::Outline;
use crate::types::model::Paragraph;
use crate::types::util::UUIDv7Base64URL;
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
    include_paragraphs: bool,
}

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_relation<R: Runtime>(
    app_handle: AppHandle<R>,
    outline_ids: Vec<UUIDv7Base64URL>,
    paragraph_ids: Vec<UUIDv7Base64URL>,
    option: RelationOption,
) -> eyre::Result<(Vec<Outline>, Vec<Paragraph>)> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let (outline_ids, paragraph_ids) = match option.include_children {
        Some(opt) => fetch::descendant_ids(pool, &outline_ids, opt.include_paragraphs).await?,
        None => (outline_ids, paragraph_ids),
    };

    let (outlines, paragraphs) = match option.direction {
        Direction::Back => fetch::relation_back(pool, &outline_ids, &paragraph_ids).await,
        Direction::Forward => fetch::relation_forward(pool, &outline_ids, &paragraph_ids).await,
    }?;

    eyre::Ok((outlines, paragraphs))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::create_version::test::create_version;
    use crate::commands::upsert_paragraph::test::upsert_paragraph;
    use crate::database::test::create_mock_user_and_pot;
    use crate::database::test::create_tree;
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;

    #[test]
    fn test_fetch_relation() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            let (_, pot) = create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle, pot.id).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>, pot_id: UUIDv7Base64URL) {
        let pool = get_state::<MockRuntime, SqlitePool>(app_handle).unwrap();

        let r1 = create_tree(app_handle, pot_id, None, 3, 0).await;
        let r2 = create_tree(app_handle, pot_id, None, 3, 0).await;

        let p1 = Paragraph::new(r1.id, None);
        upsert_paragraph(app_handle, pot_id, &p1, vec![])
            .await
            .unwrap();

        let p2 = Paragraph::new(r2.id, None);
        upsert_paragraph(app_handle, pot_id, &p2, vec![])
            .await
            .unwrap();

        let p3 = Paragraph::new(r1.id, None);
        upsert_paragraph(app_handle, pot_id, &p3, vec![])
            .await
            .unwrap();

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
                INSERT INTO paragraph_links (id_from, id_to)
                VALUES (?, ?);
            "#,
            p1.id,
            r2.id
        )
        .execute(pool)
        .await
        .unwrap();

        let version_id = UUIDv7Base64URL::new();
        create_version(app_handle.clone(), pot_id, version_id)
            .await
            .unwrap();

        sqlx::query!(
            r#"
                INSERT INTO quotes (paragraph_id, quoted_id, version_id, doc)
                VALUES (?, ?, ?, ?), (?, ?, ?, ?);
            "#,
            p1.id,
            p2.id,
            version_id,
            "",
            p2.id,
            p3.id,
            version_id,
            "",
        )
        .execute(pool)
        .await
        .unwrap();

        let (outlines, paragraphs) = fetch_relation(
            app_handle.clone(),
            vec![r2.id],
            vec![],
            RelationOption {
                direction: Direction::Back,
                include_children: Some(IncludeChildrenOption {
                    include_paragraphs: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(paragraphs.len(), 2);

        let (outlines, paragraphs) = fetch_relation(
            app_handle.clone(),
            vec![r1.id],
            vec![],
            RelationOption {
                direction: Direction::Back,
                include_children: Some(IncludeChildrenOption {
                    include_paragraphs: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(paragraphs.len(), 1);

        let (outlines, paragraphs) = fetch_relation(
            app_handle.clone(),
            vec![r1.id],
            vec![],
            RelationOption {
                direction: Direction::Forward,
                include_children: Some(IncludeChildrenOption {
                    include_paragraphs: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 1);
        assert_eq!(paragraphs.len(), 1);

        let (outlines, paragraphs) = fetch_relation(
            app_handle.clone(),
            vec![r2.id],
            vec![],
            RelationOption {
                direction: Direction::Forward,
                include_children: Some(IncludeChildrenOption {
                    include_paragraphs: true,
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(outlines.len(), 0);
        assert_eq!(paragraphs.len(), 1);
    }
}
