use crate::database::query;
use crate::types::model::Breadcrumb;
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Runtime;

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn fetch_breadcrumbs<R: Runtime>(
    app_handle: AppHandle<R>,
    parent_ids: Vec<Base64>,
) -> anyhow::Result<Vec<Breadcrumb>> {
    let pool = app_handle
        .try_state::<SqlitePool>()
        .ok_or(anyhow!("failed to get SqlitePool"))?
        .inner();

    query::fetch_breadcrumbs(pool, parent_ids).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::query;
    use crate::database::test::create_mock_user_and_pot;
    use crate::test::run_in_mock_app;
    use crate::types::model::Outline;
    use crate::types::state::AppState;
    use crate::types::util::NullableBase64;
    use std::sync::RwLock;
    use tauri::test::MockRuntime;
    use tauri::AppHandle;

    #[test]
    fn test_fetch_breadcrumbs() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
            create_mock_user_and_pot(app_handle.clone()).await;
            test(app_handle).await;
        });
    }

    async fn test(app_handle: &AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();
        let lock = app_handle.state::<RwLock<AppState>>().inner();

        let pot_id = {
            let app_state = lock.read().unwrap();
            let pot = app_state.pot.as_ref().unwrap();
            pot.id.clone()
        };

        let root = Outline::new(None);
        query::insert_outline(pool, &root, &pot_id).await.unwrap();

        let child = Outline::new(Some(&root.id));
        query::insert_outline(pool, &child, &pot_id).await.unwrap();

        let grand_child = Outline {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            parent_id: NullableBase64::from(child.id.clone()),
            fractional_index: String::new(),
            text: Some(String::new()),
        };

        query::insert_outline(pool, &grand_child, &pot_id)
            .await
            .unwrap();

        let parent_id = grand_child.parent_id.inner().unwrap().clone();

        let breadcrumbs = fetch_breadcrumbs(app_handle.clone(), vec![parent_id])
            .await
            .unwrap();
        assert_eq!(breadcrumbs.len(), 2);
    }
}
