mod sqlite_interface;
mod tantivy_interface;
mod utils;

use specta_typescript::Typescript;
use tauri::async_runtime;
use tauri_specta::{collect_commands, collect_events, Builder, Event};

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
            sqlite_interface::query::select_outline,
            sqlite_interface::query::insert_outline::<tauri::Wry>,
            sqlite_interface::query::select_cards,
            sqlite_interface::query::insert_card::<tauri::Wry>,
            tantivy_interface::index,
            tantivy_interface::search
        ])
        .events(collect_events![
            sqlite_interface::table::TableChangeEvent<sqlite_interface::table::CardsTable>,
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
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let app_handle = app.handle();

            let sqlite_handle = {
                let app_handle = app_handle.clone();
                async_runtime::spawn(async move {
                    sqlite_interface::init_sqlite(Some(&app_handle)).await
                })
            };

            let tantivy_handle = {
                let app_handle = app_handle.clone();
                async_runtime::spawn(async move {
                    tantivy_interface::init_tantivy(Some(&app_handle)).await
                })
            };

            async_runtime::block_on(sqlite_handle)??;
            async_runtime::block_on(tantivy_handle)??;

            sqlite_interface::table::TableChangeEvent::<sqlite_interface::table::CardsTable>::listen(app_handle, |e| {
                dbg!(e.payload);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
