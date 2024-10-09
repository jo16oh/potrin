use crate::database::table::{
    CardChangeEvent, CardYUpdateChangeEvent, CardYUpdatesTableChangeEvent, CardsTableChangeEvent,
    OutlineChangeEvent, OutlineYUpdateChangeEvent, OutlineYUpdatesTableChangeEvent,
    OutlinesTableChangeEvent,
};

pub fn events() -> tauri_specta::Events {
    tauri_specta::collect_events![
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
