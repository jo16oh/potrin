use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use anyhow::anyhow;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager, Runtime, State};

// pub fn lock_state_read<'a, T>(
//     state: &'a State<'_, RwLock<T>>,
// ) -> anyhow::Result<RwLockReadGuard<'a, T>>
// where
//     T: Sync + Send,
// {
//     state.read().map_err(|e| anyhow!(e.to_string()))
// }
//
// pub fn lock_state_write<'a, T>(
//     state: &'a State<'_, RwLock<T>>,
// ) -> anyhow::Result<RwLockWriteGuard<'a, T>>
// where
//     T: Sync + Send,
// {
//     state.write().map_err(|e| anyhow!(e.to_string()))
// }
//
pub async fn set_rw_state<R: Runtime, T>(app_handle: &AppHandle<R>, value: T) -> anyhow::Result<()>
where
    T: 'static + Sync + Send,
{
    match app_handle.try_state::<RwLock<T>>() {
        Some(ref mut state) => {
            let mut state = state.write().await;
            *state = value;
        }
        None => {
            app_handle.manage(RwLock::new(value));
        }
    };
    Ok(())
}

pub fn get_rw_state<R: Runtime, T>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<State<'_, RwLock<T>>>
where
    T: 'static + Sync + Send,
{
    app_handle
        .try_state::<RwLock<T>>()
        .ok_or(anyhow!("failed to get state"))
}

pub fn get_state<R: Runtime, T>(app_handle: &AppHandle<R>) -> anyhow::Result<State<'_, T>>
where
    T: 'static + Sync + Send,
{
    app_handle
        .try_state::<T>()
        .ok_or(anyhow!("failed to get state"))
}
