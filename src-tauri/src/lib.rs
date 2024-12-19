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
use tauri::{
    async_runtime, App, Manager, Runtime, TitleBarStyle, WebviewUrl, WebviewWindowBuilder
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
// use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(commands::commands())
        .events(events::events())
        .error_handling(tauri_specta::ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number),
            "../src/generated/tauri-commands.ts",
        )
        .expect("Failed to export typescript bindings");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // .plugin(
        //     tauri_plugin_log::Builder::new()
        //         .targets([
        //             Target::new(TargetKind::Stdout),
        //             Target::new(TargetKind::LogDir { file_name: None }),
        //             Target::new(TargetKind::Webview),
        //         ])
        //         .build(),
        // )
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
          setup(specta_builder, app)?;

          let app_handle = app.app_handle();

          let window_handle = async_runtime::spawn({
            let app_handle = app_handle.clone();
            async move { window::init_windows(&app_handle).await}
          });
          async_runtime::block_on(window_handle)??;

          Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(not(target_os = "macos"))]
                {
                    event.window().hide().unwrap();
                }

                #[cfg(target_os = "macos")]
                {
                    tauri::AppHandle::hide(window.app_handle()).unwrap();
                }
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::Exit => {
            app_handle.save_window_state(StateFlags::all()).unwrap();
            println!("exit");
        }
        tauri::RunEvent::ExitRequested { code, api, .. } => {
            println!("{:?}", code);

            api.prevent_exit();
        }
        _ => {}
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
