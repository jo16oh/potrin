use crate::database::query::fetch;
use crate::types::model::{Outline, Paragraph};
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use chrono::{DateTime, Duration};
use eyre::OptionExt;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    from: i64,
) -> eyre::Result<(Vec<Outline>, Vec<Paragraph>)> {
    let from = DateTime::from_timestamp_millis(from).ok_or_eyre("invalid timestamp")?;
    let to = from + Duration::days(1);

    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let paragraphs = fetch::paragraphs_by_created_at(pool, from, to).await?;
    let outline_ids: Vec<UUIDv7Base64URL> = paragraphs.iter().map(|c| c.outline_id).collect();
    let outlines = fetch::outlines_by_id(pool, &outline_ids).await?;

    eyre::Ok((outlines, paragraphs))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_mock_pot;
    use crate::database::test::create_tree;
    use crate::test::run_in_mock_app;
    use chrono::Local;
    use tauri::test::MockRuntime;

    #[test]
    fn test() {
        run_in_mock_app!(test_impl);
    }

    async fn test_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot = create_mock_pot(app_handle).await;

        create_tree(app_handle, pot.id, None, 2, 0).await;

        let start_of_day = Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .to_utc()
            .timestamp_millis();

        let (outlines, paragraphs) = fetch_timeline(app_handle.clone(), start_of_day)
            .await
            .unwrap();

        assert_eq!(outlines.len(), 3);
        assert_eq!(paragraphs.len(), 3);

        Ok(())
    }
}
