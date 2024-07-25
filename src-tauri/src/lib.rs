use tantivy_interface::SearchResults;
use tauri::AppHandle;

mod tantivy_interface;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn tantivy_init(app_handle: AppHandle) -> Result<(), String> {
    tantivy_interface::init(app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
fn tantivy_index(json: &str) -> Result<(), String> {
    tantivy_interface::index(json).map_err(|e| e.to_string())
}

#[tauri::command]
fn tantivy_search(
    input: &str,
    levenshtein_distance: u8,
    limit: usize,
) -> Result<SearchResults, String> {
    tantivy_interface::search(input, levenshtein_distance, limit).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            tantivy_init,
            tantivy_index,
            tantivy_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
