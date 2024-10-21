mod delete_card_logically;
mod delete_card_physically;
mod delete_outline_logically;
mod delete_outline_physically;
mod fetch_breadcrumbs;
mod fetch_relation;
mod fetch_relation_count;
mod fetch_timeline;
mod fetch_tree;
mod insert_card;
mod insert_outline;
mod insert_pot;
mod insert_tree_version;
mod insert_user;
mod search;
mod update_app_state;
mod update_card;
mod update_outline;

pub use delete_card_logically::*;
pub use delete_card_physically::*;
pub use delete_outline_logically::*;
pub use delete_outline_physically::*;
pub use fetch_breadcrumbs::*;
pub use fetch_relation::*;
pub use fetch_relation_count::*;
pub use fetch_timeline::*;
pub use fetch_tree::*;
pub use insert_card::*;
pub use insert_outline::*;
pub use insert_pot::*;
pub use insert_tree_version::*;
pub use insert_user::*;
pub use search::*;
pub use update_app_state::*;
pub use update_card::*;
pub use update_outline::*;

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
        update_card::<tauri::Wry>,
        update_outline::<tauri::Wry>,
        fetch_tree::<tauri::Wry>,
        fetch_timeline::<tauri::Wry>,
        fetch_relation::<tauri::Wry>,
        fetch_relation_count::<tauri::Wry>,
        fetch_breadcrumbs::<tauri::Wry>,
        delete_card_physically::<tauri::Wry>,
        delete_outline_physically::<tauri::Wry>,
        delete_card_logically::<tauri::Wry>,
        delete_outline_logically::<tauri::Wry>,
        search::<tauri::Wry>,
        update_app_state::<tauri::Wry>,
        insert_tree_version::<tauri::Wry>
    ]
}
