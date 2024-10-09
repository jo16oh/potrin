mod commands;
mod database;
mod events;
mod search_engine;
mod state;
mod types;
mod utils;

use specta_typescript::Typescript;
use tauri::{async_runtime, App, Runtime};

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

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| setup(specta_builder, app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup<R: Runtime>(
    builder: tauri_specta::Builder<R>,
    app: &mut App<R>,
) -> Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
    builder.mount_events(app);
    let app_handle = app.handle();

    let handle = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { database::init(&app_handle).await }
    });

    let handle2 = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { search_engine::init(&app_handle, 0).await }
    });

    async_runtime::block_on(handle)??;

    let handle3 = async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move { state::init(app_handle).await }
    });

    async_runtime::block_on(handle2)??;
    async_runtime::block_on(handle3)??;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    pub use crate::run_in_mock_app;
    pub use std::boxed::Box;
    pub use std::panic;
    pub use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    pub use std::sync::Arc;
    pub use std::thread;
    pub use tauri::test::MockRuntime;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    pub use tauri::{async_runtime, App, AppHandle, Manager};

    pub fn mock_app() -> App<MockRuntime> {
        let specta_builder = tauri_specta::Builder::<MockRuntime>::new().events(events::events());
        mock_builder()
            .invoke_handler(specta_builder.invoke_handler())
            .setup(move |app| setup(specta_builder, app))
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    #[macro_export]
    macro_rules! run_in_mock_app {
            (|$arg:ident: $arg_type:ty| async $closure:block) => {{
                let is_successful = Arc::new(AtomicBool::new(false));

                panic::set_hook(Box::new(move |panic_info| {
                    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                        if *s == "SUCCESS" {
                            return
                        } else if *s == "assertion failed: is_successful.load(SeqCst)" {
                            return
                        }
                    }

                    if let Some(location) = panic_info.location() {
                        println!("panic occurred in file '{}' at line {}",
                            location.file(),
                            location.line(),
                        );
                    } else {
                        println!("panic occurred but couldn't get location information...");
                    }

                    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                        println!("{}", s);
                    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                        println!("{}", s);
                    }
                }));

                {
                    let is_successful = is_successful.clone();
                    let handle = thread::spawn(|| {
                        std::panic::catch_unwind(|| {
                            let mock_app = mock_app();
                            mock_app.run(move |$arg: $arg_type, _event| {
                                async_runtime::block_on(async {
                                    $closure
                                    is_successful.store(true, SeqCst);
                                });
                                // suppress unused variable warnings
                                let _ = $arg;
                                panic!("SUCCESS");
                            })
                        })
                    });
                    let _ = handle.join();
                }

                assert!(is_successful.load(SeqCst));
            }};
        }
}
