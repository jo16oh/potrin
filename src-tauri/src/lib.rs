mod sqlite_interface;
mod tantivy_interface;
pub mod utils;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::{async_runtime, Manager};
use tauri_specta::{collect_commands, Builder};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            greet,
            sqlite_interface::insert,
            sqlite_interface::select,
            sqlite_interface::select_all,
            tantivy_interface::index,
            tantivy_interface::search
        ])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number),
            "../src/generated/tauri-commands.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let path = Arc::new(app_handle.path().app_data_dir()?);

            let sqlite_path = Arc::clone(&path);
            let sqlite_handle = async_runtime::spawn(async move {
                sqlite_interface::init_sqlite(Some(&*sqlite_path)).await
            });

            let tantivy_path = Arc::clone(&path);
            let tantivy_handle = async_runtime::spawn(async move {
                tantivy_interface::init_tantivy(Some(&*tantivy_path)).await
            });

            let _ = async_runtime::block_on(sqlite_handle)??;
            let _ = async_runtime::block_on(tantivy_handle)??;

            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
