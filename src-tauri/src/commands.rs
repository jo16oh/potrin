use crate::types::util::UUIDv7Base64;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
pub mod create_pot;
pub mod create_user;
pub mod create_version;
pub mod fetch_breadcrumbs;
pub mod fetch_conflicting_outline_ids;
pub mod fetch_pots;
pub mod fetch_relation;
pub mod fetch_relation_count;
pub mod fetch_timeline;
pub mod fetch_tree;
pub mod fetch_y_updates_by_doc_id;
pub mod get_app_state;
pub mod get_workspace_state;
pub mod hard_delete_y_doc;
pub mod insert_pending_y_update;
pub mod search;
pub mod soft_delete_card;
pub mod soft_delete_outline;
pub mod update_app_state;
pub mod update_workspace_state;
pub mod upsert_card;
pub mod upsert_outline;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn create_child(app_handle: AppHandle) {
    let main = app_handle.get_webview_window("main").unwrap();

    let sub_window =
        WebviewWindowBuilder::new(&app_handle, "sub", tauri::WebviewUrl::App("/".into()))
            .title("Settings")
            .visible(true);

    sub_window.parent(&main).unwrap().build().unwrap();
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
async fn open_pot(
    app_handle: AppHandle,
    pot_id: UUIDv7Base64,
    pot_name: String,
) -> anyhow::Result<()> {
    crate::window::open_pot(&app_handle, pot_id, pot_name).await
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
fn open_pot_selector(app_handle: AppHandle) -> anyhow::Result<()> {
    crate::window::open_pot_selector(&app_handle)
}

pub fn commands() -> tauri_specta::Commands<tauri::Wry> {
    tauri_specta::collect_commands![
        create_child,
        greet,
        create_user::create_user::<tauri::Wry>,
        create_pot::create_pot::<tauri::Wry>,
        upsert_outline::upsert_outline::<tauri::Wry>,
        upsert_card::upsert_card::<tauri::Wry>,
        create_version::create_version::<tauri::Wry>,
        insert_pending_y_update::insert_pending_y_update::<tauri::Wry>,
        soft_delete_card::soft_delete_card::<tauri::Wry>,
        soft_delete_outline::soft_delete_outline::<tauri::Wry>,
        hard_delete_y_doc::hard_delete_outline::<tauri::Wry>,
        hard_delete_y_doc::hard_delete_card::<tauri::Wry>,
        fetch_pots::fetch_pots::<tauri::Wry>,
        fetch_tree::fetch_tree::<tauri::Wry>,
        fetch_timeline::fetch_timeline::<tauri::Wry>,
        fetch_relation::fetch_relation::<tauri::Wry>,
        fetch_relation_count::fetch_relation_count::<tauri::Wry>,
        fetch_breadcrumbs::fetch_breadcrumbs::<tauri::Wry>,
        fetch_y_updates_by_doc_id::fetch_y_updates_by_doc_id::<tauri::Wry>,
        fetch_conflicting_outline_ids::fetch_conflicting_outline_ids::<tauri::Wry>,
        search::search::<tauri::Wry>,
        get_app_state::get_app_state,
        update_app_state::update_app_state::<tauri::Wry>,
        get_workspace_state::get_workspace_state,
        update_workspace_state::update_workspace_state::<tauri::Wry>,
        open_pot,
        open_pot_selector
    ]
}
