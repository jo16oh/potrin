pub mod app_version;
pub mod create_pot;
pub mod create_user;
pub mod create_version;
pub mod fetch_conflicting_outline_ids;
pub mod fetch_path;
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
pub mod open_pot;
pub mod open_pot_selector;
pub mod search;
pub mod update_app_state;
pub mod update_workspace_state;
pub mod upsert_outline;
pub mod upsert_paragraph;

pub fn commands() -> tauri_specta::Commands<tauri::Wry> {
    tauri_specta::collect_commands![
        // create_user::create_user::<tauri::Wry>,
        create_pot::create_pot::<tauri::Wry>,
        upsert_outline::upsert_outline::<tauri::Wry>,
        upsert_paragraph::upsert_paragraph::<tauri::Wry>,
        create_version::create_version::<tauri::Wry>,
        insert_pending_y_update::insert_pending_y_update::<tauri::Wry>,
        hard_delete_y_doc::hard_delete_outline::<tauri::Wry>,
        hard_delete_y_doc::hard_delete_paragraph::<tauri::Wry>,
        fetch_pots::fetch_pots::<tauri::Wry>,
        fetch_tree::fetch_tree::<tauri::Wry>,
        fetch_timeline::fetch_timeline::<tauri::Wry>,
        fetch_relation::fetch_relation::<tauri::Wry>,
        fetch_relation_count::fetch_relation_count::<tauri::Wry>,
        fetch_path::fetch_path::<tauri::Wry>,
        fetch_y_updates_by_doc_id::fetch_y_updates_by_doc_id::<tauri::Wry>,
        fetch_conflicting_outline_ids::fetch_conflicting_outline_ids::<tauri::Wry>,
        search::search::<tauri::Wry>,
        get_app_state::get_app_state,
        update_app_state::update_app_state::<tauri::Wry>,
        get_workspace_state::get_workspace_state,
        update_workspace_state::update_workspace_state::<tauri::Wry>,
        open_pot::open_pot,
        open_pot_selector::open_pot_selector,
        app_version::app_version,
    ]
}
