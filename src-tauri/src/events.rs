use crate::types::model::{CardChangeEvent, OutlineChangeEvent};

pub fn events() -> tauri_specta::Events {
    tauri_specta::collect_events![OutlineChangeEvent, CardChangeEvent,]
}
