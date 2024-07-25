mod tantivy_interface;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            tantivy_interface::init(&app_handle).unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            tantivy_interface::index,
            tantivy_interface::search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
