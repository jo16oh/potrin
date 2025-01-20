use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
pub fn app_version(app_handle: AppHandle) -> String {
    app_handle.package_info().version.to_string()
}
