use crate::database::query::fetch;
use crate::types::model::TimelineDay;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::get_state;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
pub enum FetchTimelineOption {
    Latest,
    At(i64),
    Before(i64),
    After(i64),
}

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn fetch_timeline<R: Runtime>(
    app_handle: AppHandle<R>,
    option: FetchTimelineOption,
) -> eyre::Result<Option<TimelineDay>> {
    let pool = get_state::<R, SqlitePool>(&app_handle)?;

    let from = match option {
        FetchTimelineOption::Latest => {
            let latest = sqlx::query_scalar!(
                r#"
                    SELECT MAX(created_at) AS latest
                    FROM paragraphs;
                "#
            )
            .fetch_one(pool)
            .await?;

            match latest {
                Some(ts) => get_day_start(ts),
                None => return eyre::Ok(None),
            }
        }
        FetchTimelineOption::At(ts) => get_day_start(ts),
        FetchTimelineOption::Before(ts) => {
            let day_start_of_the_day = get_day_start(ts).timestamp_millis();

            let latest_ts_before_the_day = sqlx::query_scalar!(
                r#"
                    SELECT MAX(created_at) AS "latest: i64"
                    FROM paragraphs
                    WHERE created_at < ?;
                "#,
                day_start_of_the_day
            )
            .fetch_optional(pool)
            .await?
            .flatten();

            match latest_ts_before_the_day {
                Some(ts) => get_day_start(ts),
                None => return eyre::Ok(None),
            }
        }
        FetchTimelineOption::After(ts) => {
            let day_start_after_the_day =
                (get_day_start(ts) + Duration::days(1)).timestamp_millis();

            let nearest_ts_after_the_day = sqlx::query_scalar!(
                r#"
                    SELECT MIN(created_at) AS "latest: i64"
                    FROM paragraphs
                    WHERE ? <= created_at;
                "#,
                day_start_after_the_day
            )
            .fetch_optional(pool)
            .await?
            .flatten();

            match nearest_ts_after_the_day {
                Some(ts) => get_day_start(ts),
                None => return eyre::Ok(None),
            }
        }
    };

    let to = from + Duration::days(1);

    let paragraphs = fetch::paragraphs_by_created_at(pool, from, to).await?;

    if paragraphs.is_empty() {
        return eyre::Ok(None);
    };

    let outline_ids: Vec<UUIDv7Base64URL> = paragraphs.iter().map(|c| c.outline_id).collect();
    let outlines = fetch::outlines_by_id(pool, &outline_ids).await?;

    eyre::Ok(Some(TimelineDay {
        day_start: from.timestamp_millis(),
        paragraphs,
        outlines,
    }))
}

pub fn get_day_start(unix_ts_millis: i64) -> DateTime<Utc> {
    let seconds = unix_ts_millis / 1000;

    let dt_utc = Utc
        .timestamp_opt(seconds, 0)
        .single()
        .expect("Invalid timestamp");

    let dt_local = dt_utc.with_timezone(&Local);

    Local
        .with_ymd_and_hms(dt_local.year(), dt_local.month(), dt_local.day(), 0, 0, 0)
        .single()
        .expect("Invalid date")
        .to_utc()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::database::test::create_mock_pot;
    use crate::database::test::create_tree;
    use crate::test::run_in_mock_app;
    use tauri::test::MockRuntime;

    #[test]
    fn test() {
        run_in_mock_app!(test_impl);
    }

    async fn test_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot = create_mock_pot(app_handle).await;

        create_tree(app_handle, pot.id, None, 2, 0).await;

        let res = fetch_timeline(app_handle.clone(), FetchTimelineOption::Latest)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(res.outlines.len(), 3);
        assert_eq!(res.paragraphs.len(), 3);

        let res = fetch_timeline(
            app_handle.clone(),
            FetchTimelineOption::At((Utc::now()).timestamp_millis()),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(res.outlines.len(), 3);
        assert_eq!(res.paragraphs.len(), 3);

        let res = fetch_timeline(
            app_handle.clone(),
            FetchTimelineOption::Before((Utc::now() + Duration::days(2)).timestamp_millis()),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(res.outlines.len(), 3);
        assert_eq!(res.paragraphs.len(), 3);

        let res = fetch_timeline(
            app_handle.clone(),
            FetchTimelineOption::After((Utc::now() - Duration::days(2)).timestamp_millis()),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(res.outlines.len(), 3);
        assert_eq!(res.paragraphs.len(), 3);

        Ok(())
    }
}
