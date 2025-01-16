pub use crate::run_in_mock_app;

use super::*;
use tauri::test::MockRuntime;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::App;

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
        let is_successful = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        std::panic::set_hook(std::boxed::Box::new(move |panic_info| {
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
            let handle = std::thread::spawn(|| {
                std::panic::catch_unwind(|| {
                    let mock_app = $crate::test::mock_app();
                    mock_app.run(move |$arg: $arg_type, _event| {
                        tauri::async_runtime::block_on(async {
                            $closure
                            is_successful.store(true, std::sync::atomic::Ordering::SeqCst);
                        });
                        // suppress unused variable warnings
                        let _ = $arg;
                        panic!("SUCCESS");
                    })
                })
            });
            let _ = handle.join();
        }

        assert!(is_successful.load(std::sync::atomic::Ordering::SeqCst));
    }};
}
