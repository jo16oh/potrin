use crate::events::AppStateChange;
use crate::search_engine::SearchIndex;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{get_rw_state, get_state};
use crate::{types::state::*, utils::set_rw_state};
use json_patch::Patch;
use sqlx::SqlitePool;
use tauri::{AppHandle, EventTarget, Manager, Runtime, WebviewWindow, Window};
use tauri_specta::Event;

struct QueryResult {
    value: Vec<u8>,
}

pub async fn init_app_state<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let prev_state = sqlx::query_as!(
        QueryResult,
        r#"
            SELECT value
            FROM kvs
            WHERE id = ?;
        "#,
        "app_state"
    )
    .fetch_optional(pool)
    .await?;

    let app_state = match prev_state {
        Some(result) => serde_sqlite_jsonb::from_slice::<AppState>(&result.value)?,
        None => {
            let app_state = AppState::default();

            let jsonb = serde_sqlite_jsonb::to_vec(&app_state)?;

            sqlx::query!(
                r#"
                    INSERT INTO kvs (id, value)
                    VALUES (?, ?)
                    ON CONFLICT DO UPDATE
                    SET
                        value = excluded.value;
                "#,
                "app_state",
                jsonb
            )
            .execute(pool)
            .await?;

            app_state
        }
    };

    set_rw_state::<R, AppState>(app_handle, app_state).await?;

    Ok(())
}

pub async fn init_workspace_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &WebviewWindow<R>,
    pot_id: UUIDv7Base64URL,
    pot_name: &str,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;

    let prev_state = sqlx::query_as!(
        QueryResult,
        r#"
            SELECT value
            FROM workspaces
            WHERE pot_id = ?;
        "#,
        pot_id
    )
    .fetch_optional(pool)
    .await?;

    let workspace_state = match prev_state {
        Some(result) => serde_sqlite_jsonb::from_slice::<WorkspaceState>(&result.value)?,
        None => {
            let app_state = WorkspaceState::new(pot_id, pot_name.to_string());

            let jsonb = serde_sqlite_jsonb::to_vec(&app_state)?;

            sqlx::query!(
                r#"
                    INSERT INTO workspaces (pot_id, value)
                    VALUES (?, ?)
                    ON CONFLICT DO UPDATE
                    SET
                        value = excluded.value;
                "#,
                pot_id,
                jsonb
            )
            .execute(pool)
            .await?;

            app_state
        }
    };

    set_rw_state::<R, WorkspaceState>(window, workspace_state).await?;

    Ok(())
}

pub async fn update_app_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    patch: String,
    origin_window_label: &str,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let mut app_state = lock.write().await;
    let patch_deserealized = &serde_json::from_str::<Patch>(&patch)?;

    let current_app_state = {
        let mut value = serde_json::to_value(&*app_state)?;
        json_patch::patch(&mut value, patch_deserealized)?;
        serde_json::from_value::<AppState>(value)?
    };

    let current_app_state_jsonb = serde_sqlite_jsonb::to_vec(&current_app_state)?;

    sqlx::query!(
        r#"
            UPDATE kvs
            SET value = ?
            WHERE id = "app_state";
        "#,
        current_app_state_jsonb
    )
    .execute(pool)
    .await?;

    *app_state = current_app_state;

    AppStateChange::new(patch).emit_filter(app_handle, |target| match target {
        EventTarget::WebviewWindow { label } => label != origin_window_label,
        EventTarget::Webview { label } => label != origin_window_label,
        EventTarget::Window { label } => label != origin_window_label,
        _ => true,
    })?;

    Ok(())
}

pub async fn update_workspace_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &Window<R>,
    patch: String,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, WorkspaceState>(window)?;

    let mut workspace_state = lock.write().await;
    let patch_serealized = &serde_json::from_str::<Patch>(&patch)?;

    let current_workspace_state = {
        let mut value = serde_json::to_value(&*workspace_state)?;
        json_patch::patch(&mut value, patch_serealized)?;
        serde_json::from_value::<WorkspaceState>(value)?
    };

    let current_app_state_jsonb = serde_sqlite_jsonb::to_vec(&current_workspace_state)?;

    sqlx::query!(
        r#"
            UPDATE workspaces
            SET value = ?
            WHERE pot_id = ?;
        "#,
        current_app_state_jsonb,
        current_workspace_state.pot.id
    )
    .execute(pool)
    .await?;

    *workspace_state = current_workspace_state;

    Ok(())
}

pub async fn close_pot<R: Runtime>(
    app_handle: &AppHandle<R>,
    window: &Window<R>,
    pot_id: &UUIDv7Base64URL,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle)?;
    let lock = get_rw_state::<R, AppState>(app_handle)?;

    let mut app_state = lock.write().await;
    let prev_app_state = serde_json::to_value(app_state.clone())?;
    app_state.pots.remove(pot_id);
    let current_app_state = serde_json::to_value(app_state.clone())?;

    let patch = json_patch::diff(&prev_app_state, &current_app_state).to_string();

    let current_app_state_jsonb = serde_sqlite_jsonb::to_vec(&*app_state)?;

    sqlx::query!(
        r#"
            UPDATE kvs
            SET value = ?
            WHERE id = "app_state";
        "#,
        current_app_state_jsonb
    )
    .execute(pool)
    .await?;

    window.unmanage::<SearchIndex>();

    AppStateChange::new(patch).emit(app_handle)?;
    Ok(())
}
