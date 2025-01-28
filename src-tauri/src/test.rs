pub use crate::run_in_mock_app;

use super::*;
use tauri::test::MockRuntime;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::App;

#[test]
fn generate_specta() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(commands::commands())
        .events(events::events())
        .error_handling(tauri_specta::ErrorHandlingMode::Result);

    let result = specta_builder.export(
        Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number),
        "../src/generated/tauri-commands.ts",
    );

    assert!(result.is_ok());
}

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
    ($func:expr) => {{
        let is_successful = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        std::panic::set_hook(std::boxed::Box::new(move |panic_info| {
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                if *s == "SUCCESS" {
                    return;
                } else if *s == "FAIL" {
                    return;
                }
            }

            if let Some(location) = panic_info.location() {
                eprintln!(
                    "panic occurred in file '{}' at line {}",
                    location.file(),
                    location.line(),
                );
            } else {
                eprintln!("panic occurred but couldn't get location information");
            }

            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                println!("{}", s);
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                println!("{}", s);
            }
        }));

        {
            let is_successful = is_successful.clone();
            let handle = std::thread::spawn(|| {
                std::panic::catch_unwind(|| {
                    let mock_app = $crate::test::mock_app();
                    mock_app.run(move |app_handle, _event| {
                        tauri::async_runtime::block_on(async {
                            let result: eyre::Result<()> = $func(&app_handle).await;
                            match result {
                                Ok(_) => {
                                    is_successful.store(true, std::sync::atomic::Ordering::SeqCst);
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    panic!("FAIL");
                                }
                            }
                        });
                        panic!("SUCCESS");
                    })
                })
            });
            let _ = handle.join();
        }

        let result = is_successful.load(std::sync::atomic::Ordering::SeqCst);

        if !result {
            panic!("FAIL");
        }
    }};
}
