use crate::types::util::UUIDv7Base64URL;
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
#[macros::eyre_to_any]
#[macros::log_err]
pub async fn open_pot(app_handle: AppHandle, pot_id: UUIDv7Base64URL) -> eyre::Result<()> {
    crate::window::open_pot(&app_handle, pot_id).await
}
