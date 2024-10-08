mod database;
mod search_engine;
mod state;
mod utils;

use database::table::{
    CardChangeEvent, CardYUpdateChangeEvent, CardYUpdatesTableChangeEvent, CardsTableChangeEvent,
    OutlineChangeEvent, OutlineYUpdateChangeEvent, OutlineYUpdatesTableChangeEvent,
    OutlinesTableChangeEvent,
};
use specta_typescript::Typescript;
use tauri::{async_runtime, App, Runtime};
use tauri_specta::{collect_commands, collect_events, Events};

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
            database::query::insert_outline::<tauri::Wry>,
            database::query::insert_card::<tauri::Wry>,
            database::query::fetch_tree::<tauri::Wry>,
            database::query::fetch_timeline::<tauri::Wry>,
            database::query::fetch_relation::<tauri::Wry>,
            database::query::fetch_relation_count::<tauri::Wry>,
            search_engine::index,
            search_engine::search,
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
        .setup(move |app| setup(specta_builder, app))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn events() -> Events {
    collect_events![
        OutlinesTableChangeEvent,
        OutlineYUpdatesTableChangeEvent,
        CardsTableChangeEvent,
        CardYUpdatesTableChangeEvent,
        OutlineChangeEvent,
        OutlineYUpdateChangeEvent,
        CardChangeEvent,
        CardYUpdateChangeEvent
    ]
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
    use crate::database::query::insert_pot;
    use crate::database::table::{Pot, User};
    use crate::database::types::Base64;
    pub use crate::run_in_mock_app;
    use sqlx::SqlitePool;
    use state::types::{PotState, UserState};
    use state::update_app_state;
    pub use std::boxed::Box;
    pub use std::panic;
    pub use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    pub use std::sync::Arc;
    pub use std::thread;
    pub use tauri::test::MockRuntime;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    pub use tauri::{async_runtime, App, AppHandle, Manager};

    pub fn mock_app() -> App<MockRuntime> {
        let specta_builder = tauri_specta::Builder::<MockRuntime>::new().events(events());
        mock_builder()
            .invoke_handler(specta_builder.invoke_handler())
            .setup(move |app| setup(specta_builder, app))
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    pub async fn create_mock_user_and_pot(app_handle: AppHandle<MockRuntime>) {
        let pool = app_handle.state::<SqlitePool>().inner();

        let user = User {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            name: "mock_user".to_string(),
        };

        sqlx::query!(
            r#"
                INSERT INTO users (id, name)
                VALUES (?, ?);
            "#,
            user.id,
            user.name
        )
        .execute(pool)
        .await
        .unwrap();

        update_app_state(
            app_handle.clone(),
            state::AppStateValues::User(Some(UserState {
                id: user.id.clone().to_string(),
                name: user.name.clone(),
            })),
        )
        .await
        .unwrap();

        let pot = Pot {
            id: Base64::from(uuidv7::create_raw().to_vec()),
            name: "mock".to_string(),
            owner: user.id.clone(),
        };

        insert_pot(app_handle.clone(), pot.clone()).await.unwrap();

        update_app_state(
            app_handle.clone(),
            state::AppStateValues::Pot(Some(PotState {
                id: pot.id.to_string(),
                sync: false,
            })),
        )
        .await
        .unwrap();
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
