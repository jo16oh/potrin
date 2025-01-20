use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub fn open_pot_selector(app_handle: AppHandle) -> eyre::Result<()> {
    crate::window::open_pot_selector(&app_handle)
}
