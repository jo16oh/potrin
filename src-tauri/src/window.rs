use crate::{
    search_engine::load_index,
    state::init_workspace_state,
    types::{state::AppState, util::UUIDv7Base64},
    utils::get_rw_state,
};
use tauri::{AppHandle, Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_window_state::{StateFlags, WindowExt};

pub async fn init_windows(app_handle: &AppHandle) -> anyhow::Result<()> {
    let app_state_lock = get_rw_state::<_, AppState>(app_handle)?;
    let app_state = app_state_lock.read().await;

    if app_state.pots.is_empty() {
        open_pot_selector(app_handle)?;
    } else {
        for id in app_state.pots.iter() {
            open_pot(app_handle, *id, "".into()).await?;
        }
    }

    Ok(())
}

pub fn open_pot_selector(app_handle: &AppHandle) -> anyhow::Result<()> {
    let win_builder =
        WebviewWindowBuilder::new(app_handle, "pot-selector", WebviewUrl::App("".into()))
            .title("Potrin")
            .hidden_title(true)
            .inner_size(800.0, 600.0);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);
    win_builder.build()?;

    Ok(())
}

pub async fn open_pot(
    app_handle: &AppHandle,
    pot_id: UUIDv7Base64,
    pot_name: String,
) -> anyhow::Result<()> {
    let win_builder = WebviewWindowBuilder::new(
        app_handle,
        "pot-selector",
        WebviewUrl::App(format!("pot/{}", pot_id).into()),
    )
    .title(&pot_name)
    .hidden_title(true)
    .inner_size(800.0, 600.0);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);

    let window = win_builder.build()?;

    window.restore_state(StateFlags::all())?;

    init_workspace_state(app_handle).await?;

    let search_index = load_index(app_handle, pot_id, 0).await?;
    window.manage(search_index);

    Ok(())
}
