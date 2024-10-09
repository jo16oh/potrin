use crate::types::model::{
    CardChangeEvent, CardYUpdateChangeEvent, OutlineChangeEvent, OutlineYUpdateChangeEvent,
};

pub fn events() -> tauri_specta::Events {
    tauri_specta::collect_events![
        OutlineChangeEvent,
        OutlineYUpdateChangeEvent,
        CardChangeEvent,
        CardYUpdateChangeEvent
    ]
}
