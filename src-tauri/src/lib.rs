mod tantivy_interface;
pub mod utils;

use specta_typescript::Typescript;
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
            tantivy_interface::init,
            tantivy_interface::index,
            tantivy_interface::search
        ])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/generated/tauri-commands.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
