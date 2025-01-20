mod commands;
mod database;
mod events;
mod reconciler;
mod search_engine;
mod state;
mod types;
mod utils;
mod window;

#[cfg(test)]
mod test;

use specta_typescript::Typescript;
use state::close_pot;
use tauri::{async_runtime, App, Manager, Runtime};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use types::util::UUIDv7Base64URL;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(commands::commands())
        .events(events::events())
        .error_handling(tauri_specta::ErrorHandlingMode::Result);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number),
            "../src/generated/tauri-commands.ts",
        )
        .expect("Failed to export typescript bindings");

    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all() - StateFlags::VISIBLE)
                .with_denylist(&["pot-selector"])
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            app.get_webview_window("main")
                .expect("no main window")
                .set_focus()
                .unwrap();
        }))
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            setup(specta_builder, app)?;

            let app_handle = app.app_handle();

            let window_handle = async_runtime::spawn({
                let app_handle = app_handle.clone();
                async move { window::init_windows(&app_handle).await }
            });
            async_runtime::block_on(window_handle)??;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() != "pot-selector" {
                    let app_handle = window.app_handle();
                    let pot_id: UUIDv7Base64URL = window.label().try_into().unwrap();

                    if app_handle.webview_windows().len() > 1 {
                        let task_handle = async_runtime::spawn({
                            let app_handle = app_handle.clone();
                            async move {
                                close_pot(&app_handle, &pot_id).await.unwrap();
                            }
                        });
                        async_runtime::block_on(task_handle).unwrap();
                    }
                }

                window
                    .app_handle()
                    .save_window_state(StateFlags::all())
                    .unwrap();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            app_handle.save_window_state(StateFlags::all()).unwrap();
        }
    });
}

fn setup<R: Runtime>(
    builder: tauri_specta::Builder<R>,
    app: &mut App<R>,
) -> Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
    builder.mount_events(app);
    let app_handle = app.handle();

    let database_handle = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { database::init(&app_handle).await }
    });
    async_runtime::block_on(database_handle)??;

    let app_state_handle = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { state::init_app_state(&app_handle).await }
    });
    async_runtime::block_on(app_state_handle)??;

    let reconciler_handle = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { reconciler::init(&app_handle).await }
    });
    async_runtime::block_on(reconciler_handle)??;

    Ok(())
}
