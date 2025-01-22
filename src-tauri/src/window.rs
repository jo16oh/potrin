use crate::{
    search_engine::load_index,
    state::init_workspace_state,
    types::{setting::SearchFuzziness, state::AppState, util::UUIDv7Base64URL},
    utils::get_rw_state,
};
use tauri::{AppHandle, Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

pub async fn init_windows(app_handle: &AppHandle) -> eyre::Result<()> {
    let app_state_lock = get_rw_state::<_, AppState>(app_handle)?;
    let app_state = app_state_lock.read().await;

    if app_state.pots.is_empty() {
        open_pot_selector(app_handle)?;
    } else {
        for (id, name) in app_state.pots.iter() {
            open_pot(app_handle, *id, name).await?;
        }
    }

    Ok(())
}

pub fn open_pot_selector(app_handle: &AppHandle) -> eyre::Result<()> {
    let win_builder =
        WebviewWindowBuilder::new(app_handle, "pot-selector", WebviewUrl::App("".into()))
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

pub async fn open_pot(
    app_handle: &AppHandle,
    pot_id: UUIDv7Base64URL,
    pot_name: &str,
) -> eyre::Result<()> {
    let win_builder = WebviewWindowBuilder::new(
        app_handle,
        pot_id,
        WebviewUrl::App(format!("pot/{}", pot_id).into()),
    )
    .title(pot_name)
    .hidden_title(true)
    .inner_size(1025.0, 800.0)
    .visible(false);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);

    let window = win_builder.build()?;

    init_workspace_state(app_handle, &window, pot_id, pot_name)
        .await
        .unwrap();

    let search_index = load_index(app_handle, pot_id, SearchFuzziness::Exact).await?;
    window.manage(search_index);

    Ok(())
}
