mod sqlite_interface;
mod tantivy_interface;
mod utils;

use specta_typescript::Typescript;
use sqlite_interface::table::*;
use tauri::{async_runtime, App, Runtime};
use tauri_specta::{collect_commands, collect_events, Events};
use Env::*;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[allow(dead_code)]
#[derive(Clone)]
enum Env {
    Build,
    Test,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            greet,
            sqlite_interface::query::select_outline,
            sqlite_interface::query::insert_outline::<tauri::Wry>,
            sqlite_interface::query::select_cards,
            sqlite_interface::query::insert_card::<tauri::Wry>,
            tantivy_interface::index,
            tantivy_interface::search
        ])
        .events(events())
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
        .setup(move |app| setup(specta_builder, app, Build))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn events() -> Events {
    collect_events![
        OutlinesTableChangeEvent,
        OutlineYUpdatesTableChangeEvent,
        CardsTableChangeEvent,
        CardYUpdatesTableChangeEvent,
    ]
}

fn setup<R: Runtime>(
    builder: tauri_specta::Builder<R>,
    app: &mut App<R>,
    env: Env,
) -> Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
    builder.mount_events(app);
    let app_handle = app.handle();

    let sqlite_handle = {
        let app_handle = app_handle.clone();
        async_runtime::spawn(async move { sqlite_interface::init_sqlite(&app_handle).await })
    };

    let tantivy_handle = {
        let app_handle = app_handle.clone();
        async_runtime::spawn(async move { tantivy_interface::init_tantivy(&app_handle).await })
    };

    // set event listners here
    // OutlinesTableChangeEvent::listen(app_handle, |e| {
    //     dbg!(e.payload);
    // });

    async_runtime::block_on(sqlite_handle)??;
    async_runtime::block_on(tantivy_handle)??;

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
    pub use tauri::{async_runtime, App};

    pub fn mock_app() -> App<MockRuntime> {
        let specta_builder = tauri_specta::Builder::<MockRuntime>::new().events(events());
        mock_builder()
            .invoke_handler(specta_builder.invoke_handler())
            .setup(move |app| setup(specta_builder, app, Test))
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    #[macro_export]
    macro_rules! run_in_mock_app {
            (|$arg:ident: $arg_type:ty| async $closure:block) => {{
                let is_successful = Arc::new(AtomicBool::new(false));

                {
                    let is_successful = Arc::clone(&is_successful);
                    panic::set_hook(Box::new(move |panic_info| {
                        if !is_successful.load(SeqCst) {
                            if let Some(location) = panic_info.location() {
                                println!("panic occurred in file '{}' at line {}",
                                    location.file(),
                                    location.line(),
                                );
                            } else {
                                println!("panic occurred but can't get location information...");
                            }

                            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                                println!("{s:?}");
                            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                                println!("{s:?}");
                            } else {
                                println!("panic occurred");
                            }
                        };
                    }));
                }

                {
                    let is_successful = is_successful.clone();
                    let handle = thread::spawn(|| {
                        println!("thread running");
                        std::panic::catch_unwind(|| {
                            let mock_app = mock_app();
                            mock_app.run(move |$arg: $arg_type, _event| {
                                async_runtime::block_on(async {
                                    $closure
                                    is_successful.store(true, SeqCst);
                                });
                                $arg.exit(1);
                            })
                        })
                    });
                    let _ = handle.join();
                }


                assert!(is_successful.load(SeqCst));
            }};
        }
}
