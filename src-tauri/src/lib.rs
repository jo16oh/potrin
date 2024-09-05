mod sqlite_interface;
mod tantivy_interface;
mod utils;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use tauri::{async_runtime, App, Runtime};
use tauri_specta::{collect_commands, collect_events, Event, Events};

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DemoEvent(pub String);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
        .setup(move |app| setup(specta_builder, app, Env::Build))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Clone)]
enum Env {
    Test,
    Build,
}

fn events() -> Events {
    collect_events![
        sqlite_interface::table::TableChangeEvent<sqlite_interface::table::OutlinesTable>,
        DemoEvent,
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
        let env = env.clone();
        async_runtime::spawn(async move {
            sqlite_interface::init_sqlite(match env {
                Env::Build => Some(&app_handle),
                Env::Test => None,
            })
            .await
        })
    };

    let tantivy_handle = {
        let app_handle = app_handle.clone();
        let env = env.clone();
        async_runtime::spawn(async move {
            tantivy_interface::init_tantivy(match env {
                Env::Build => Some(&app_handle),
                Env::Test => None,
            })
            .await
        })
    };

    async_runtime::block_on(sqlite_handle)??;
    async_runtime::block_on(tantivy_handle)??;

    sqlite_interface::table::TableChangeEvent::<sqlite_interface::table::OutlinesTable>::listen(
        app_handle,
        |e| {
            dbg!(e.payload);
        },
    );

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    pub use crate::run_in_mock_app;
    use tauri::{
        test::{mock_builder, mock_context, noop_assets, MockRuntime},
        App,
    };

    pub fn mock_app() -> App<MockRuntime> {
        let specta_builder = tauri_specta::Builder::<MockRuntime>::new().events(events());
        mock_builder()
            .invoke_handler(specta_builder.invoke_handler())
            .setup(move |app| setup(specta_builder, app, Env::Test))
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    #[macro_export]
    macro_rules! run_in_mock_app {
        (|$arg:ident: $arg_type:ty| async $closure:block) => {{

            panic::set_hook(Box::new(|_| {}));

            let is_successfull = Arc::new(AtomicBool::new(false));

            let flag_clone = is_successfull.clone();
            let handle = thread::spawn(|| {
                println!("thread running");
                panic::catch_unwind(|| {
                    let mock_app = mock_app();
                    mock_app.run(move |$arg: $arg_type, _event| {
                        async_runtime::block_on(async {
                            $closure
                            flag_clone.store(true, Ordering::SeqCst);
                        });
                        $arg.exit(1);
                    })
                })
            });

            let _ = handle.join();

            assert!(is_successfull.load(Ordering::SeqCst));
        }};
    }
}
