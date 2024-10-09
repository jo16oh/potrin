mod fetch_breadcrumbs;
mod fetch_relation;
mod fetch_relation_count;
mod fetch_timeline;
mod fetch_tree;
mod index;
mod insert_card;
mod insert_outline;
mod insert_pot;
mod insert_user;
mod search;
mod update_app_state;

pub use fetch_breadcrumbs::*;
pub use fetch_relation::*;
pub use fetch_relation_count::*;
pub use fetch_timeline::*;
pub use fetch_tree::*;
pub use index::*;
pub use insert_card::*;
pub use insert_outline::*;
pub use insert_pot::*;
pub use insert_user::*;
pub use search::*;
pub use update_app_state::*;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn commands() -> tauri_specta::Commands<tauri::Wry> {
    tauri_specta::collect_commands![
        greet,
        insert_user::<tauri::Wry>,
        insert_pot::<tauri::Wry>,
        insert_outline::<tauri::Wry>,
        insert_card::<tauri::Wry>,
        fetch_tree::<tauri::Wry>,
        fetch_timeline::<tauri::Wry>,
        fetch_relation::<tauri::Wry>,
        fetch_relation_count::<tauri::Wry>,
        fetch_breadcrumbs::<tauri::Wry>,
        index,
        search,
        update_app_state::<tauri::Wry>
    ]
}
