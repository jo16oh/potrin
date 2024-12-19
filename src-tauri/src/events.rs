use crate::types::{
    model::{CardForIndex, OutlineForIndex},
    util::{BytesBase64, UUIDv7Base64},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Init,
    Remote,
    Local { window_label: String },
}

impl Origin {
    pub fn local(label: &str) -> Self {
        Origin::Local {
            window_label: label.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct AppStateChange {
    pub patch: String,
}

impl AppStateChange {
    pub fn new(patch: String) -> Self {
        AppStateChange { patch }
    }
}

#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct WorkspaceStateChange {
    pub patch: String,
}

impl WorkspaceStateChange {
    pub fn new(patch: String) -> Self {
        WorkspaceStateChange { patch }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct Target<T> {
    pub current_value: T,
    pub related_y_updates: Vec<BytesBase64>,
}

impl<T> Target<T> {
    pub fn new(current_value: T, related_y_updates: Vec<BytesBase64>) -> Self {
        Target {
            current_value,
            related_y_updates,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub enum Operation<T> {
    Insert { targets: Vec<Target<T>> },
    Update { targets: Vec<Target<T>> },
    Delete { target_ids: Vec<UUIDv7Base64> },
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct OutlineChange {
    operation: Operation<OutlineForIndex>,
    origin: Origin,
}

impl OutlineChange {
    pub fn insert(targets: Vec<Target<OutlineForIndex>>, origin: Origin) -> Self {
        OutlineChange {
            operation: Operation::Insert { targets },
            origin,
        }
    }

    pub fn update(targets: Vec<Target<OutlineForIndex>>, origin: Origin) -> Self {
        OutlineChange {
            operation: Operation::Update { targets },
            origin,
        }
    }

    pub fn delete(target_ids: Vec<UUIDv7Base64>, origin: Origin) -> Self {
        OutlineChange {
            operation: Operation::Delete { target_ids },
            origin,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CardChange {
    operation: Operation<CardForIndex>,
    origin: Origin,
}

impl CardChange {
    pub fn insert(targets: Vec<Target<CardForIndex>>, origin: Origin) -> Self {
        CardChange {
            operation: Operation::Insert { targets },
            origin,
        }
    }

    pub fn update(targets: Vec<Target<CardForIndex>>, origin: Origin) -> Self {
        CardChange {
            operation: Operation::Update { targets },
            origin,
        }
    }

    pub fn delete(target_ids: Vec<UUIDv7Base64>, origin: Origin) -> Self {
        CardChange {
            operation: Operation::Delete { target_ids },
            origin,
        }
    }
}

pub fn events() -> tauri_specta::Events {
    tauri_specta::collect_events![
        AppStateChange,
        WorkspaceStateChange,
        OutlineChange,
        CardChange
    ]
}
