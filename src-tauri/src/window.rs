use crate::{
    database::query::fetch,
    state::{init_window_state, update_app_state},
    types::{state::AppState, util::UUIDv7Base64URL},
    utils::{get_rw_state, get_state},
};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

pub async fn init_windows(app_handle: &AppHandle) -> eyre::Result<()> {
    let pots = {
        let app_state_lock = get_rw_state::<_, AppState>(app_handle)?;
        let app_state = app_state_lock.read().await;
        app_state.pots.clone()
    };

    if pots.is_empty() {
        open_pot_selector(app_handle)?;
    } else {
        for (id, _) in pots.iter() {
            open_pot(app_handle, *id).await?;
        }
    }

    Ok(())
}

pub fn open_pot_selector(app_handle: &AppHandle) -> eyre::Result<()> {
    if let Some(win) = app_handle.webview_windows().get("entry") {
        return win.set_focus().map_err(|e| e.into());
    }

    let win_builder = WebviewWindowBuilder::new(app_handle, "entry", WebviewUrl::App("".into()))
        .title("Potrin")
        .hidden_title(true)
        .resizable(false)
        .inner_size(800.0, 650.0)
        .visible(false);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);
    win_builder.build()?;

    Ok(())
}

pub async fn open_pot(app_handle: &AppHandle, pot_id: UUIDv7Base64URL) -> eyre::Result<()> {
    if let Some(win) = app_handle.webview_windows().get(&pot_id.to_string()) {
        return win.set_focus().map_err(|e| e.into());
    }

    let pool = get_state::<_, SqlitePool>(app_handle)?;

    let pot = fetch::pot_by_id(pool, pot_id).await?;

    let win_builder = WebviewWindowBuilder::new(
        app_handle,
        pot.id,
        WebviewUrl::App(format!("pot/{}", pot.id).into()),
    )
    .title(&pot.name)
    .hidden_title(true)
    .inner_size(1025.0, 800.0)
    .min_inner_size(470.0, 380.0)
    .visible(false);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);

    win_builder.build()?;

    init_window_state(app_handle, &pot).await?;

    let app_state = {
        let app_state_lock = get_rw_state::<_, AppState>(app_handle)?;
        let mut app_state = app_state_lock.read().await.clone();
        app_state.pots.insert(pot.id, pot.name);
        app_state
    };

    update_app_state(app_handle, app_state, "").await?;

    Ok(())
}
