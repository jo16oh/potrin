use super::query::RawOutline;
use serde_json::from_str;
use tauri::{AppHandle, Listener};

pub fn start_sync(app_handle: &AppHandle) {
    app_handle.listen("data_change", |event| {
        let payload = event.payload();
        let res = from_str::<RawOutline>(payload);
        dbg!(res);
    });
}
